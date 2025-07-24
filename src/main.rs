#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod query;
mod platform;

use std::{error::Error, fs};
use directories::ProjectDirs;
use serde::Deserialize;
use clap::Parser;
use crate::{platform::{nogui, HideAndSeek}, query::weather::get_weather};

#[cfg(not(feature = "headless"))]
slint::include_modules!();

const CITY: &str = "Oulu";
const COUNTRY: &str = "FI";

#[derive(Parser, Debug, Clone)]
#[command(version, about = "Taskbar-Weather — a tool to fetch your local weather. Copyright © 2025 Markku Sukanen. See LICENSE.")]
struct CommandLineArgs {
    /// If given, doesn't use UI and logs to console instead.
    #[arg(long, )]
    headless: bool,
    /// If given, checkes weather once, logs it to console, and quits right after.
    #[arg(short, long, )]
    pub oneshot: bool,
}

#[derive(Deserialize)]
struct Config {
    city: String,
    country: String,
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
    fn adjust_to_features(self) -> Self;
}

#[cfg(not(feature = "headless"))]
impl ArgsAdjuster for CommandLineArgs {
    fn adjust_to_features(self) -> Self {self}
}
#[cfg(feature = "headless")]
impl ArgsAdjuster for CommandLineArgs {
    fn adjust_to_features(self) -> Self {
        let mut args = self.clone();
        args.headless = true;
        args
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = env_logger::try_init();

    let args = CommandLineArgs::parse().adjust_to_features();

    let config = if let Some(proj_dirs) = ProjectDirs::from("net", "msukanen", "TaskbarWeather") {
        let config_path = proj_dirs.config_dir().join("config.toml");
        
        if !config_path.exists() {
            log::debug!("No config file - creating an example config.toml …");
            let default_config_content = format!(
                "city = \"{}\"\ncountry = \"{}\"\n",
                CITY, COUNTRY
            );
            if let Some(parent_dir) = config_path.parent() {
                fs::create_dir_all(parent_dir).ok();
            }
            fs::write(&config_path, default_config_content).ok();
        }

        fs::read_to_string(config_path)
            .ok()
            .and_then(|content| toml::from_str::<Config>(&content).ok())
    } else {
        log::warn!("Something went seriously wrong with config …");
        None
    };

    // Use the config values if they exist (they should — if the config
    // read/write didn't miserably fail), otherwise use the defaults.
    let (city, country) = if let Some(conf) = config {
        log::debug!("Config read, city set as '{}' and country as '{}'.", conf.city, conf.country);
        (conf.city, conf.country)
    } else {
        (CITY.to_string(), COUNTRY.to_string())
    };

    if args.oneshot {
        nogui::get_weather(&city, &country).await;
        return Ok(());
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
