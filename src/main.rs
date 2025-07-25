#![cfg_attr(all(not(debug_assertions), not(feature="headless"), target_os="windows"), windows_subsystem = "windows")]
#![cfg_attr(all(not(debug_assertions), feature="headless", target_os="windows"), windows_subsystem = "console")]
mod query;
mod platform;

use std::{error::Error, fs, time::Duration};
use directories::ProjectDirs;
use serde::Deserialize;
use clap::Parser;
use crate::platform::nogui;
#[cfg(not(feature = "headless"))]
use crate::{platform::HideAndSeek, query::weather::get_weather};

#[cfg(not(feature = "headless"))]
slint::include_modules!();

const CITY: &str = "Oulu";
const COUNTRY: &str = "FI";
const MILLIS_CHECK_DELAY: u64 = 900_000; // 15 minutes (± a few picoseconds). NOTE: do keep this about samey as in ui/overlay.slint.

#[derive(Parser, Debug, Clone)]
#[command(version, about = "Taskbar-Weather — a tool to fetch your local weather. See LICENSE.")]
struct CommandLineArgs {
    /// If given, doesn't use UI and logs to console instead.
    #[arg(long, )]
    headless: bool,
    /// If given, checkes weather once, logs it to console, and quits right after.
    #[arg(short, long, )]
    oneshot: bool,
    /// If given, overrides configured city setting(s).
    #[arg(long, )]
    city: Option<String>,
    /// If given, overrides configured country setting(s).
    #[arg(long, )]
    country: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    city: Option<String>,
    country: Option<String>,
}

#[cfg(not(feature = "headless"))]
async fn fetch_and_update_weather(weak: slint::Weak<Overlay>, city: String, country: String) {
    let result = get_weather(&city, &country).await;
    slint::invoke_from_event_loop(move || {
        if let Some(handle) = weak.upgrade() {
            let temp_text = match result {
                Ok(json) => json["main"]["temp"]
                    .as_f64()
                    .map(|t| format!("{:.1}°C in {}", t, city))
                    .unwrap_or_else(|| "Weather unavailable.".into()),
                Err(_) => "Weather unavailable.".into(),
            };
            handle.set_temperature_text(temp_text.into());
        }
    }).unwrap();
}

trait ArgsAdjuster {
    /// Adjust args to compile time features.
    fn adjust_to_features(self) -> Self;
}

/// Enforce full uppercase on country code/name.
macro_rules! args_adjust_uc_country {
    ($args:expr) => {
        if let Some(country) = $args.country {
            $args.country = Some(country.to_ascii_uppercase());
        }
    };
}

#[cfg(not(feature = "headless"))]
impl ArgsAdjuster for CommandLineArgs {
    fn adjust_to_features(self) -> Self {
        let mut args = self.clone();
        args_adjust_uc_country!(args);
        args
    }
}
#[cfg(feature = "headless")]
impl ArgsAdjuster for CommandLineArgs {
    fn adjust_to_features(self) -> Self {
        let mut args = self.clone();
        args_adjust_uc_country!(args);
        args.headless = true;
        args
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(target_os="windows")]
    {
        // We want possible --headless/--oneshot call to actually print in PS/cmd, so …:
        use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
        unsafe {
            let _ = AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }
    let _ = env_logger::try_init();

    let args = CommandLineArgs::parse().adjust_to_features();
    let mut new_config = false;

    let config = if let Some(proj_dirs) = ProjectDirs::from("net", "msukanen", "TaskbarWeather") {
        let config_path = proj_dirs.config_dir().join("config.toml");
        
        if !config_path.exists() {
            log::debug!("No config file - creating an example config.toml …");
            let default_config_content = format!(
                "# Set your (favorite?) city and country here:\n#city = \"{}\"\n#country = \"{}\"\n",
                CITY, COUNTRY
            );
            if let Some(parent_dir) = config_path.parent() {
                fs::create_dir_all(parent_dir).ok();
            }
            fs::write(&config_path, default_config_content).ok();
            eprintln!("NOTE: configuration file established, do edit/uncomment the appropriate values in\n  {}\n", config_path.display());
            new_config = true;
        }

        fs::read_to_string(config_path)
            .ok()
            .and_then(|content| toml::from_str::<Config>(&content).ok())
    } else {
        None
    };

    // City - either args.city, config.city, or default (which most likely is wrong…).
    let city = args.city.clone().or_else(|| config.as_ref()?.city.clone());
    let country = args.country.clone().or_else(|| config.as_ref()?.country.clone());

    if city.is_none() || country.is_none() {
        if new_config {
            eprintln!("Location not yet concretely configured, please edit the above mentioned configuration file.");
        } else {
            eprintln!("Error: location is not (yet) concretely configured.");

            if let Some(proj_dirs) = ProjectDirs::from("net", "msukanen", "TaskbarWeather") {
                let config_path = proj_dirs.config_dir().join("config.toml");
                eprintln!("\nTo fix this, please create or edit the config file at:\n  {}", config_path.display());
                eprintln!("\nAnd add your location, for example:\n  city = \"YourCity\"\n  country = \"XY\"")
            }
        }
        eprintln!("\nAlternatively, specify a location using command-line arguments.\nRun with  --help  to see all the available options and their usage.");
        std::process::exit(1);
    }
    let city = city.unwrap();
    let country = country.unwrap();

    log::info!("City set as '{}'", city);
    log::info!("Country set as '{}'", country);

    if args.oneshot {
        if let Ok(info) = nogui::get_weather(&city, &country).await {
            println!("{}", info);
            return Ok(());
        }
        std::process::exit(1);
    }

    if args.headless {
        loop {
            if let Ok(info) = nogui::get_weather(&city, &country).await {
                println!("{}", info);
            }
            tokio::time::sleep(Duration::from_millis(MILLIS_CHECK_DELAY)).await;
        }
    }

    #[cfg(not(feature = "headless"))]
    {
        if !args.headless {
            let ui = Overlay::new()?;
            ui.stealth();

            let weak = ui.as_weak();
            log::info!("Initial weather fetch …");
            tokio::spawn(fetch_and_update_weather(weak.clone(), city.clone(), country.clone()));

            // --- Set up the timer callback ---
            ui.on_request_weather_update(move || {
                let weak = weak.clone();
                let city = city.clone();
                let country = country.clone();
                log::debug!("Re-fetching weather data …");
                tokio::spawn(fetch_and_update_weather(weak, city, country));
            });

            ui.run()?;
        }
    }
    Ok(())
}
