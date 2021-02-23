use crate::WeatherReport;
use reqwest::header::CONTENT_TYPE;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct OpenWeather {
    api_key: String,
    api_path_prefix: String
}

#[derive(Debug)]
struct OpenWeatherJsonParseError;
impl Error for OpenWeatherJsonParseError {}
impl fmt::Display for OpenWeatherJsonParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to build weather report from open weather response json")
    }
}

const API_PATH_PREFIX : &str = "http://api.openweathermap.org/data/2.5";
impl OpenWeather {
    pub fn new(api_key: String) -> Self {
        Self { api_key: api_key, api_path_prefix: API_PATH_PREFIX.to_string() }
    }

    #[cfg(test)]
    pub fn new_with_prefix(api_key: String, api_path_prefix: String) -> Self {
        Self { api_key: api_key, api_path_prefix: api_path_prefix }
    }

    pub async fn get_current(&self, city_name: &str) -> Result<WeatherReport, Box<dyn Error>> {
        let full_path = format!("{}/weather?APPID={}&q={}&units=metric",
                                self.api_path_prefix, self.api_key, city_name);
        let raw_json = Self::get_raw(full_path).await?;
        Self::parse_report_from_raw_json(raw_json)
    }

    pub async fn get_forecast(&self, city_name: &str, days_count: usize) -> Result<Vec<WeatherReport>, Box<dyn Error>> {
        let full_path = format!("{}/forecast?APPID={}&q={}&cnt={}&units=metric",
                                self.api_path_prefix, self.api_key, city_name, days_count);
        let raw_json = Self::get_raw(full_path).await?;
        Self::parse_report_array_from_raw_json(raw_json)
    }

    async fn get_raw(full_path: String) -> Result<serde_json::Value, Box<dyn Error>> {
        let client = reqwest::Client::new();
        let raw_result = client
            .get(&full_path)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?
            .json()
            .await?;

        Ok(raw_result)
    }

    fn parse_report_from_raw_json(data: serde_json::Value) -> Result<WeatherReport, Box<dyn Error>> {
        Self::parse_report_from_open_weather_json_struct(&data)
    }

    fn parse_report_array_from_raw_json(data: serde_json::Value) -> Result<Vec<WeatherReport>, Box<dyn Error>> {
        let array = data["list"].as_array();
        if array.is_some() {
            array.unwrap().into_iter().map(Self::parse_report_from_open_weather_json_struct).collect()
        } else {
            Err(OpenWeatherJsonParseError.into())
        }
    }

    fn parse_report_from_open_weather_json_struct(data: &serde_json::Value) -> Result<WeatherReport, Box<dyn Error>> {
        let temp = data["main"]["temp"].as_f64();
        let timestamp = data["dt"].as_i64();
        if temp.is_some() && timestamp.is_some() {
            Ok(WeatherReport { temperature: temp.unwrap(), unix_timestamp: timestamp.unwrap() })
        } else {
            Err(OpenWeatherJsonParseError.into())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::MockServer;
    use httpmock::Method::GET;

    #[test]
    fn it_deserializes_current_weather_valid_raw_json() {
        let raw_json = r#"
        {
            "main": {
                "temp": -12.94
            },
            "dt": 1613978904,
            "name": "Moscow"
        }
        "#;
        let json_value = serde_json::from_str(raw_json).unwrap();

        let weather_report = OpenWeather::parse_report_from_raw_json(json_value).unwrap();
        assert_eq!(weather_report.temperature, -12.94);
        assert_eq!(weather_report.unix_timestamp, 1613978904);
    }

    #[test]
    fn it_fails_to_deserialize_current_weather_invalid_raw_json() {
        let raw_json = r#"
        {
            "main": {
                "xtemp": -12.94
            },
            "dt": 1613978904,
            "name": "Moscow"
        }
        "#;
        let json_value = serde_json::from_str(raw_json).unwrap();

        assert_eq!(
            OpenWeather::parse_report_from_raw_json(json_value).is_err(),
            true
        )
    }


    #[test]
    fn it_deserializes_forecast_weather_valid_raw_json() {
        let raw_json = r#"
        {
            "list": [
                {
                    "main": {
                        "temp": -13.45
                    },
                    "dt": 1613984400
                },
                {
                    "main": {
                        "temp": -13.21
                    },
                    "dt": 1613995200
                }
            ],
            "city": {
                "name": "Moscow"
            }
        }
        "#;
        let json_value = serde_json::from_str(raw_json).unwrap();

        let parsed_reports = OpenWeather::parse_report_array_from_raw_json(json_value).unwrap();

        assert_eq!(parsed_reports[0].temperature, -13.45);
        assert_eq!(parsed_reports[0].unix_timestamp, 1613984400);
        assert_eq!(parsed_reports[1].temperature, -13.21);
        assert_eq!(parsed_reports[1].unix_timestamp, 1613995200);
    }

    #[test]
    fn it_fails_to_deserialize_forecast_weather_invalid_raw_json() {
        let raw_json = r#"
        {
            "list": [
                {
                    "main": {
                        "temsp": -13.45
                    },
                    "dt": 1613984400
                },
                {
                    "main": {
                        "temp": -13.21
                    },
                    "dt": 1613995200
                }
            ],
            "city": {
                "name": "Moscow"
            }
        }
        "#;
        let json_value = serde_json::from_str(raw_json).unwrap();

        assert_eq!(
            OpenWeather::parse_report_array_from_raw_json(json_value).is_err(),
            true
        )
    }

    #[actix_rt::test]
    async fn it_fetches_data_from_open_weather_service() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET)
                .path("/weather");

            let json = std::fs::read_to_string("./tests/fixtures/open_weather_current_success.json").unwrap();
            then.status(200)
                .header("Content-Type", "application/json")
                .body(json);
        });

        let key = "apikey".to_string();
        let report = OpenWeather::new_with_prefix(key, server.url("")).get_current("kazan").await;

        assert_eq!(report.is_ok(), true);
        assert_eq!(report.unwrap().temperature, -26.0);
    }

    #[actix_rt::test]
    async fn it_returns_error_for_wrong_key() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET)
                .path("/weather");

            let json = std::fs::read_to_string("./tests/fixtures/open_weather_invalid_key.json").unwrap();
            then.status(401)
                .header("Content-Type", "application/json")
                .body(json);
        });

        let key = "apikey".to_string();
        let report = OpenWeather::new_with_prefix(key, server.url("")).get_current("kazan").await;

        assert_eq!(report.is_err(), true);
    }
}

