#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod query;
use std::error::Error;

use crate::query::weather::get_weather;
slint::include_modules!();

const CITY: &str = "Oulu";
const COUNTRY: &str = "FI";

async fn fetch_and_update_weather(weak: slint::Weak<Overlay>, city: String, country: String) {
    let api_key = match std::env::var("OPENWEATHER_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            slint::invoke_from_event_loop(move || {
                if let Some(handle) = weak.upgrade() {
                    handle.set_temperature_text("API Key Missing".into());
                }
            }).unwrap();
            return;
        }
    };

    let result = get_weather(&city, &country, &api_key).await;

    // The rest of the logic is the same...
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
            // Give the application a full second to initialize and settle down.

            use std::time::Duration;
            log::debug!("La-di-da-da - waiting a while…");
            thread::sleep(Duration::from_millis(1000));

            let title_wide: Vec<u16> = "TaskbarWeatherAppThigyDoohickey"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            // After waiting, we only need to try once.
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
        });
    }

    let weak = ui.as_weak();
    let city = std::env::var("TASKBARKWEATHER_CITY").map_or(CITY.to_string(), |x|x);
    let country = std::env::var("TASKBARWEATHER_COUNTRYCODE").map_or(COUNTRY.to_string(), |x|x);

    // --- Initial weather update ---
    tokio::spawn(fetch_and_update_weather(weak.clone(), city.clone(), country.clone()));

    // --- Set up the timer callback ---
    ui.on_request_weather_update(move || {
        let weak = weak.clone();
        let city = city.clone();
        let country = country.clone();
        tokio::spawn(fetch_and_update_weather(weak, city, country));
    });

    ui.run()?;
    Ok(())
}
