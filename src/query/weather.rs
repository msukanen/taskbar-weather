use std::{error::Error, fmt::Display};

pub async fn get_weather<S: Display>(
    city: S,
    country: S,
) -> Result<serde_json::Value, Box<dyn Error + Send>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("https://msukanen.net/api/weather?city={}&country={}", city, country))
        .send()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
    Ok(resp)
}
