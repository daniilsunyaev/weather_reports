use std::error::Error;
use actix_web::{web, get, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::dev::Server;
use dotenv::dotenv;

pub mod weather_clients; // TODO: remove
pub mod handlers;

#[derive(Debug)]
pub struct WeatherReport {
    pub temperature: f64
}

pub fn run() -> Result<Server, std::io::Error> {
    dotenv().ok();

    let server = HttpServer::new(|| {
        App::new().service(handlers::daily)
    })
    .bind("127.0.0.1:7878")?
    .run();

    Ok(server)
}

pub async fn get_current_weather(city_name: &str) -> Result<WeatherReport, String> {
    let mut reports : Vec<WeatherReport> = vec![];

    let (open_weather_report, weatherbit_report) =
        futures::join!(
            get_open_weather_current(city_name),
            get_weatherbit_current(city_name)
        );

    if open_weather_report.is_ok() {
        reports.push(open_weather_report.unwrap());
    };
    if weatherbit_report.is_ok() {
        reports.push(weatherbit_report.unwrap());
    }

    if reports.len() > 0 {
        Ok(average_report(reports))
    } else {
        Err(String::from("Could not find weather data"))
    }
}

pub async fn get_forecast_weather(city_name: &str, days_count: usize) -> Result<Vec<WeatherReport>, String> {
    let mut reports : Vec<Vec<WeatherReport>> = vec![];

    let (open_weather_report, weatherbit_report) =
        futures::join!(
            get_open_weather_forecast(city_name, days_count),
            get_weatherbit_forecast(city_name, days_count)
        );

    if open_weather_report.is_ok() {
        reports.push(open_weather_report.unwrap());
    };
    if weatherbit_report.is_ok() {
        reports.push(weatherbit_report.unwrap());
    }

    if reports.len() > 0 {
        Ok(average_forecast_report(reports))
    } else {
        Err(String::from("Could not find weather data"))
    }
}

pub async fn get_specific_day_weather(city_name: &str, days_since: usize) -> Result<WeatherReport, String> {
    let mut report = get_forecast_weather(city_name, days_since + 1).await?;
    Ok(report.remove(days_since))
}

async fn get_open_weather_current(city_name: &str) -> Result<WeatherReport, Box<dyn Error>> {
    weather_clients::open_weather::OpenWeather::new(open_weather_api_key()).get_current(city_name).await
}

async fn get_weatherbit_current(city_name: &str) -> Result<WeatherReport, Box<dyn Error>> {
    weather_clients::weatherbit::Weatherbit::new(weatherbit_api_key()).get_current(city_name).await
}

async fn get_open_weather_forecast(city_name: &str, days_count: usize) -> Result<Vec<WeatherReport>, Box<dyn Error>> {
    weather_clients::open_weather::OpenWeather::new(open_weather_api_key()).get_forecast(city_name, days_count).await
}

async fn get_weatherbit_forecast(city_name: &str, days_count: usize) -> Result<Vec<WeatherReport>, Box<dyn Error>> {
    weather_clients::weatherbit::Weatherbit::new(weatherbit_api_key()).get_forecast(city_name, days_count).await
}

fn weatherbit_api_key() -> String {
    std::env::var("WEATHERBIT_API_KEY").unwrap()
}

fn open_weather_api_key() -> String {
    std::env::var("OPEN_WEATHER_APPID").unwrap()
}

fn average_report(reports: Vec<WeatherReport>) -> WeatherReport {
    let average_temperature =
        reports
        .iter()
        .fold(0.0, |acc, report| (acc + report.temperature)) / (reports.len() as f64);

    WeatherReport { temperature: average_temperature }
}

fn average_forecast_report(reports: Vec<Vec<WeatherReport>>) -> Vec<WeatherReport> {
    reports.iter()
        .fold(vec![0f64; reports[0].len()], |acc, provider_forecast| {
            acc.iter()
                .zip(provider_forecast.iter())
                .map(|(sum_temp, provider_daily_report)| sum_temp + provider_daily_report.temperature)
                .collect()
        })
    .iter().map(|sum_temperature| WeatherReport { temperature: sum_temperature / (reports.len() as f64) })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn averages_several_weather_reports() {
        let reports = vec![
            WeatherReport { temperature: 2.0 },
            WeatherReport { temperature: 4.0 },
            WeatherReport { temperature: 2.0 },
            WeatherReport { temperature: 8.0 }
        ];

        assert_eq!(average_report(reports).temperature, 4.0);
    }

    #[test]
    fn averages_single_weather_report() {
        let reports = vec![
            WeatherReport { temperature: 2.0 },
        ];

        assert_eq!(average_report(reports).temperature, 2.0);
    }

    #[test]
    fn averages_several_weather_forecast_reports() {
        let reports = vec![
            vec![
                WeatherReport { temperature: 4.0 },
                WeatherReport { temperature: 4.0 },
                WeatherReport { temperature: 2.0 }
            ],
            vec![
                WeatherReport { temperature: 4.0 },
                WeatherReport { temperature: 4.0 },
                WeatherReport { temperature: 4.0 }
            ],
            vec![
                WeatherReport { temperature: 6.0 },
                WeatherReport { temperature: 4.0 },
                WeatherReport { temperature: 6.0 }
            ],
            vec![
                WeatherReport { temperature: 6.0 },
                WeatherReport { temperature: 4.0 },
                WeatherReport { temperature: 2.0 }
            ]
        ];

        let average_report = average_forecast_report(reports);

        assert_eq!(average_report[0].temperature, 5.0);
        assert_eq!(average_report[1].temperature, 4.0);
        assert_eq!(average_report[2].temperature, 3.5);
    }

    #[test]
    fn averages_single_weather_forecast_report() {
        let reports = vec![
            vec![WeatherReport { temperature: 2.0 }, WeatherReport { temperature: 3.0 }]
        ];

        let average_report = average_forecast_report(reports);

        assert_eq!(average_report[0].temperature, 2.0);
        assert_eq!(average_report[1].temperature, 3.0);
    }
}
