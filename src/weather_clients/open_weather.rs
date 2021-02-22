use crate::WeatherReport;
use reqwest::header::CONTENT_TYPE;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct OpenWeather {
    api_key: String
}

#[derive(Debug)]
struct OpenWeatherJsonParseError;
impl Error for OpenWeatherJsonParseError {}
impl fmt::Display for OpenWeatherJsonParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to build weather report from open weather response json")
    }
}

impl OpenWeather {
    const API_PATH_PREFIX: &'static str = "http://api.openweathermap.org/data/2.5";

    pub fn new(api_key: String) -> Self {
        Self { api_key: api_key }
    }

    pub async fn get_current(&self, city_name: &str) -> Result<WeatherReport, Box<dyn Error>> {
        let full_path = format!("{}/weather?APPID={}&q={}&units=metric",
                                OpenWeather::API_PATH_PREFIX, self.api_key, city_name);
        let raw_json = Self::get_raw(full_path).await?;
        Self::parse_report_from_raw_json(raw_json)
    }

    pub async fn get_forecast(&self, city_name: &str, days_count: usize) -> Result<Vec<WeatherReport>, Box<dyn Error>> {
        let full_path = format!("{}/forecast?APPID={}&q={}&cnt={}&units=metric",
                                OpenWeather::API_PATH_PREFIX, self.api_key, city_name, days_count);
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
        if temp.is_some() {
            Ok(WeatherReport { temperature: temp.unwrap() })
        } else {
            Err(OpenWeatherJsonParseError.into())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

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


        assert_eq!(
            OpenWeather::parse_report_from_raw_json(json_value).unwrap().temperature,
            -12.94
        )
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

        let parsed_reposts =OpenWeather::parse_report_array_from_raw_json(json_value).unwrap();

        assert_eq!(parsed_reposts[0].temperature, -13.45);
        assert_eq!(parsed_reposts[1].temperature, -13.21)
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
}
