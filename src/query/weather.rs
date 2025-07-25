use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum WeatherError {
    Network,
    Server(reqwest::StatusCode),
    Decode,
}

impl fmt::Display for WeatherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WeatherError::Network => write!(f, "Could not connect to the weather service. Internet in fire?"),
            WeatherError::Server(status) => write!(f, "Weather service responded with an error (Status: {}). It might be temporarily down.", status),
            WeatherError::Decode => write!(f, "Received an undecipherable response from the weather service?!"),
        }
    }
}

impl Error for WeatherError {}

pub async fn get_weather<S: fmt::Display>(
    city: S,
    country: S,
) -> Result<serde_json::Value, WeatherError> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("https://msukanen.net/api/weather?city={}&country={}", city, country))
        .send()
        .await
        .map_err(|_|WeatherError::Network)?;
    if !resp.status().is_success() {
        return Err(WeatherError::Server(resp.status()));
    }
    resp.json::<serde_json::Value>()
        .await
        .map_err(|_|WeatherError::Decode)
}
