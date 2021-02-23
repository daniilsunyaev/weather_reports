mod weather_clients;

use crate::WeatherReport;
use std::error::Error;
use weather_clients::open_weather::OpenWeather;
use weather_clients::weatherbit::Weatherbit;
use average::Mean;
use average::Estimate;

#[derive(Clone)]
pub struct AverageWeatherReport {
    pub temperature: Mean,
    pub unix_timestamp: Mean
}

impl AverageWeatherReport {
    pub fn new() -> AverageWeatherReport {
        AverageWeatherReport { temperature: Mean::new(), unix_timestamp: Mean::new() }
    }

    pub fn add(&mut self, weather_report: WeatherReport) -> &AverageWeatherReport {
        self.temperature.add(weather_report.temperature);
        self.unix_timestamp.add(weather_report.unix_timestamp as f64);
        self
    }

    pub fn mean(&self) -> WeatherReport {
        WeatherReport {
            temperature: self.temperature.mean(),
            unix_timestamp: self.unix_timestamp.mean() as i64
        }
    }
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

    if !reports.is_empty() {
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

    if !reports.is_empty() {
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
    open_weather_client().get_current(city_name).await
}

async fn get_weatherbit_current(city_name: &str) -> Result<WeatherReport, Box<dyn Error>> {
    weatherbit_client().get_current(city_name).await
}

async fn get_open_weather_forecast(city_name: &str, days_count: usize) -> Result<Vec<WeatherReport>, Box<dyn Error>> {
    open_weather_client().get_forecast(city_name, days_count).await
}

async fn get_weatherbit_forecast(city_name: &str, days_count: usize) -> Result<Vec<WeatherReport>, Box<dyn Error>> {
    weatherbit_client().get_forecast(city_name, days_count).await
}

fn open_weather_client() -> OpenWeather {
    OpenWeather::new(open_weather_api_key())
}

fn weatherbit_client() -> Weatherbit {
    Weatherbit::new(weatherbit_api_key())
}

fn weatherbit_api_key() -> String {
    std::env::var("WEATHERBIT_API_KEY").unwrap()
}

fn open_weather_api_key() -> String {
    std::env::var("OPEN_WEATHER_APPID").unwrap()
}

fn average_report(reports: Vec<WeatherReport>) -> WeatherReport {
    reports
        .into_iter()
        .fold(
            AverageWeatherReport::new(),
            |mut average, report| { average.add(report); average }
        )
        .mean()
}

fn average_forecast_report(reports: Vec<Vec<WeatherReport>>) -> Vec<WeatherReport> {
    let average_reports = vec![AverageWeatherReport::new(); reports[0].len()];
    reports
        .into_iter()
        .fold(average_reports, |average_forecast_report, forecast_report| {
            average_forecast_report.into_iter()
                .zip(forecast_report.into_iter())
                .map(|(mut average, report)| { average.add(report); average })
                .collect()
        })
        .into_iter()
        .map(|average_report| average_report.mean())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn averages_several_weather_reports() {
        let reports = vec![
            WeatherReport { temperature: 2.0, unix_timestamp: 10 },
            WeatherReport { temperature: 4.0, unix_timestamp: 20 },
            WeatherReport { temperature: 2.0, unix_timestamp: 10 },
            WeatherReport { temperature: 8.0, unix_timestamp: 20 }
        ];

        let average_report = average_report(reports);
        assert_eq!(average_report.temperature, 4.0);
        assert_eq!(average_report.unix_timestamp, 15);
    }

    #[test]
    fn averages_single_weather_report() {
        let reports = vec![
            WeatherReport { temperature: 2.0, unix_timestamp: 10 },
        ];

        let average_report = average_report(reports);
        assert_eq!(average_report.temperature, 2.0);
        assert_eq!(average_report.unix_timestamp, 10);
    }

    #[test]
    fn averages_several_weather_forecast_reports() {
        let reports = vec![
            vec![
                WeatherReport { temperature: 4.0, unix_timestamp: 10 },
                WeatherReport { temperature: 4.0, unix_timestamp: 10 },
                WeatherReport { temperature: 2.0, unix_timestamp: 10 }
            ],
            vec![
                WeatherReport { temperature: 4.0, unix_timestamp: 10 },
                WeatherReport { temperature: 4.0, unix_timestamp: 20 },
                WeatherReport { temperature: 4.0, unix_timestamp: 30 }
            ],
            vec![
                WeatherReport { temperature: 6.0, unix_timestamp: 10 },
                WeatherReport { temperature: 4.0, unix_timestamp: 20 },
                WeatherReport { temperature: 6.0, unix_timestamp: 20 }
            ],
            vec![
                WeatherReport { temperature: 6.0, unix_timestamp: 10 },
                WeatherReport { temperature: 4.0, unix_timestamp: 10 },
                WeatherReport { temperature: 2.0, unix_timestamp: 20 }
            ]
        ];

        let average_report = average_forecast_report(reports);

        assert_eq!(average_report[0].temperature, 5.0);
        assert_eq!(average_report[0].unix_timestamp, 10);
        assert_eq!(average_report[1].temperature, 4.0);
        assert_eq!(average_report[1].unix_timestamp, 15);
        assert_eq!(average_report[2].temperature, 3.5);
        assert_eq!(average_report[2].unix_timestamp, 20);
    }

    #[test]
    fn averages_single_weather_forecast_report() {
        let reports = vec![
            vec![
                WeatherReport { temperature: 2.0, unix_timestamp: 10 },
                WeatherReport { temperature: 3.0, unix_timestamp: 33 }
            ]
        ];

        let average_report = average_forecast_report(reports);

        assert_eq!(average_report[0].temperature, 2.0);
        assert_eq!(average_report[0].unix_timestamp, 10);
        assert_eq!(average_report[1].temperature, 3.0);
        assert_eq!(average_report[1].unix_timestamp, 33);
    }
}
