use crate::query::weather;

/// GUIless bridge to fetching weather info.
/// 
/// If successful, weather info is tossed into `stdout`.
/// 
/// # Arguments
/// - `city`— your location, probably, maybe.
/// - `country`— country code, e.g. "FI", "UK" (or "GB"), "US", etc.
pub async fn get_weather(city: &String, country: &String) {
    match weather::get_weather(city, country).await {
        Ok(weather) => {
            let info = format!("{:.1}⁰C (feels like {:.1}⁰C) at {}, {}",
                weather["main"]["temp"],
                weather["main"]["feels_like"],
                city, country
            );
            println!("{}", info);
            log::info!("Weather data: {}", info);
        },
        Err(e) => log::error!("Could not fetch weather data! {:?}", e)
    };
}

#[cfg(not(windows))]
impl HideAndSeek for Overlay {
    fn stealth(&self) {/* No UI = no "stealth".*/}
}
