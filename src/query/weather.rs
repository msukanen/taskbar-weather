use std::{error::Error, fmt::Display};

pub async fn get_weather<S: Display>(
    city: S,
    country: S,
    api_key: &str,
) -> Result<serde_json::Value, Box<dyn Error + Send>> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.openweathermap.org/data/2.5/weather")
        .query(&[
            ("q", format!("{},{}", city, country).as_str()),
            ("units","metric"),
            ("appid", api_key)])
        .send()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
    Ok(resp)
}
