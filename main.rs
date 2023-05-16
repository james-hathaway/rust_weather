use std::env;
use reqwest::Error;
use serde_json::Value;
use tokio;
use serde::Deserialize;


pub struct Config { // using a struct to store the URL
    pub base_url: String,
}

impl Config { //implement the config to a function, where it returns itself as the URL to a string
    pub fn new() -> Self {
        Config {
            base_url: "https://api.open-meteo.com/v1/forecast".to_string(), // Open-Meteo API URL
        }
    }
}

async fn fetch_weather_data(config: &Config, location: &str) -> Result<WeatherData, Error> { //reference config, location and return the result with error handliong
    let geonames_username = "jrh230";

    //parse the latitude and longitude
    let latitude = location.split(",").collect::<Vec<&str>>()[0]; //when the comma is input, split the two coordinates to a vector
    let longitude = location.split(",").collect::<Vec<&str>>()[1];

    //construct the URL for the GeoNames Timezone API
    let timezone_api_url = format!( // use of the format macro to make it cleaner
        "http://api.geonames.org/timezoneJSON?lat={}&lng={}&username={}",
        latitude, longitude, geonames_username
    );

    //send request to the GeoNames API
    let timezone_response = reqwest::get(&timezone_api_url).await?;
    let timezone_data: Value = timezone_response.json().await?;



    // get the timezone ID from the API response
    let timezone_id = match timezone_data.get("timezoneId") { //taking advantage of the some/none structure I learned about recently
        Some(id) => id.as_str().unwrap(),
        None => {
            println!("Failed to fetch timezone data");
            std::process::exit(1);
        }
    };
    

    let url = format!(
        "{}?latitude={}&longitude={}&timezone={}&daily=temperature_2m_min,temperature_2m_max",
        config.base_url, latitude, longitude, timezone_id
    );

    // send request to the Open-Meteo API
    let response = reqwest::get(&url).await?;
    let weather_data: WeatherData = response.json().await?;
    Ok(weather_data)

}

#[derive(Deserialize, Debug)] //thanks GPT for showing me what this stuff means and how to use it! LOL
#[allow(dead_code)]
struct WeatherData { // elegantly storing my data
    daily: DailyData,
    daily_units: DailyUnits,
    elevation: f32,
    generationtime_ms: f64,
    latitude: f32,
    longitude: f32,
    timezone: String,
    timezone_abbreviation: String,
    utc_offset_seconds: i32,
}

#[derive(Deserialize, Debug)]
struct DailyData {
    temperature_2m_max: Vec<f32>,
    temperature_2m_min: Vec<f32>,
    time: Vec<String>,
}

#[derive( Deserialize, Debug)]
#[allow(dead_code)] // threw these in when I changed the data to print out farenheit not celsius
struct DailyUnits {
    temperature_2m_max: String,
    temperature_2m_min: String,
    //time: String,
}

fn print_weather_data(location: &str, weather_data: &WeatherData) {
    println!("Weather data for {}:", location);
    println!("Timezone: {}", weather_data.timezone);
    println!("Elevation: {} meters", weather_data.elevation);
    println!("Generation Time (ms): {}", weather_data.generationtime_ms);
    println!("Latitude: {}", weather_data.latitude);
    println!("Longitude: {}", weather_data.longitude);
    println!("Timezone Abbreviation: {}", weather_data.timezone_abbreviation);
    println!("UTC Offset (seconds): {}", weather_data.utc_offset_seconds);

    for (i, time) in weather_data.daily.time.iter().enumerate() {
        println!("Date: {}", time);
        let max_temp_f = weather_data.daily.temperature_2m_max[i] * 9.0/5.0 + 32.0; // cus im american
        let min_temp_f = weather_data.daily.temperature_2m_min[i] * 9.0/5.0 + 32.0;
        println!("Max Temperature: {} F", max_temp_f);
        println!("Min Temperature: {} F", min_temp_f);
        println!("");
    }
}





#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: rusty_weather_cli <latitude,longitude>");
        std::process::exit(1);
    }

    let location = &args[1];
    let config = Config::new();
    let weather_data = match fetch_weather_data(&config, location).await {
        Ok(data) => data,
        Err(e) => {
            println!("Failed to fetch weather data: {}", e);
            std::process::exit(1);
        }
    };
    print_weather_data(location, &weather_data);
}

