use crate::query::weather::{self, WeatherError};
/// GUIless bridge to fetching weather info.
/// 
/// If successful, weather info is tossed into `stdout`.
/// 
/// # Arguments
/// - `city`— your location, probably, maybe.
/// - `country`— country code, e.g. "FI", "UK" (or "GB"), "US", etc.
pub async fn get_weather(city: &String, country: &String) -> Result<String, WeatherError> {
    let w = weather::get_weather(city, country).await;
    match w {
        Ok(weather) => {
            let info = format!("{:.1}⁰C (feels like {:.1}⁰C) at {}, {}",
                weather["main"]["temp"],
                weather["main"]["feels_like"],
                city, country
            );
            log::info!("Weather data: {}", info);
            Ok(info)
        },
        Err(e) => {
            log::error!("Could not fetch weather data! {}", e);
            Err(e)
        }
    }
}
