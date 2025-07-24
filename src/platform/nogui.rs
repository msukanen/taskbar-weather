use crate::query::weather;

pub async fn get_weather(city: &String, country: &String) {
    match weather::get_weather(city, country).await {
        Ok(weather) => {
            println!("{:.1}⁰C (feels like {:.1}⁰C) at {}, {}",
                weather["main"]["temp"],
                weather["main"]["feels_like"],
                city, country
            );
        },
        Err(e) => log::error!("Could not fetch weather data! {:?}", e)
    };
}
