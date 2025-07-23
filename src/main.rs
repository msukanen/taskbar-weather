#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod query;

use std::{error::Error, fs};
use directories::ProjectDirs;
use serde::Deserialize;
use crate::query::weather::get_weather;

slint::include_modules!();

const CITY: &str = "Oulu";
const COUNTRY: &str = "FI";
const WINDOW_TITLE: &str = "TaskbarWeatherAppThigyDoohickey"; // this has to be EXACTLY the same as it's in ui/overlay.slint

#[derive(Deserialize)]
struct Config {
    city: String,
    country: String,
}

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ui = Overlay::new()?;
    let _ = env_logger::try_init();

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

    // Use the config values if they exist, otherwise use the defaults.
    let (city, country) = if let Some(conf) = config {
        log::debug!("Config read, city set as '{}' and country as '{}'.", conf.city, conf.country);
        (conf.city, conf.country)
    } else {
        (CITY.to_string(), COUNTRY.to_string())
    };

    #[cfg(target_os = "windows")]
    {
        use std::thread;
        use windows::core::PCWSTR;
        use windows::Win32::UI::WindowsAndMessaging::{
            FindWindowW, GetDesktopWindow, GetWindowLongPtrW, SetParent, SetWindowLongPtrW,
            SetWindowPos, GWL_EXSTYLE, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE,
            SWP_NOZORDER, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
        };
        
        thread::spawn(move || {
            use std::time::Duration;
            
            log::debug!("La-di-da-da - waiting a while…");
            // Give the application a full second to initialize and settle down.
            thread::sleep(Duration::from_millis(1000));

            let title_wide: Vec<u16> = WINDOW_TITLE
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            // Window *should* be stable by now …
            log::debug!("Hum-di-dum, wait is over…");
            unsafe {
                if let Ok(slint_hwnd) = FindWindowW(PCWSTR::null(), PCWSTR::from_raw(title_wide.as_ptr())) {
                    
                    let desktop_hwnd = GetDesktopWindow();
                    SetParent(slint_hwnd, Some(desktop_hwnd)).ok();

                    let ex_style = GetWindowLongPtrW(slint_hwnd, GWL_EXSTYLE);
                    let new_ex_style = (ex_style & !(WS_EX_APPWINDOW.0 as isize)) | (WS_EX_TOOLWINDOW.0 as isize);
                    SetWindowLongPtrW(slint_hwnd, GWL_EXSTYLE, new_ex_style);

                    SetWindowPos(
                        slint_hwnd,
                        None,
                        0, 0, 0, 0,
                        SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
                    ).ok();
                    
                } else {
                    log::error!("Could not find the application window after waiting for 1 second.");
                }
            }
            log::info!("Up and running!");
        });
    }

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
    Ok(())
}
