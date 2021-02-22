use crate::WeatherReport;
use reqwest::header::CONTENT_TYPE;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct Weatherbit {
    api_key: String,
    api_path_prefix: String
}

#[derive(Debug)]
struct WeatherbitJsonParseError;
impl Error for WeatherbitJsonParseError {}
impl fmt::Display for WeatherbitJsonParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to build weather report from weatherbit response json")
    }
}

const API_PATH_PREFIX: &str = "http://api.weatherbit.io/v2.0/";

impl Weatherbit {

    pub fn new(api_key: String) -> Self {
        Self { api_key: api_key, api_path_prefix: API_PATH_PREFIX.to_string() }
    }

    // used for tests
    pub fn new_with_prefix(api_key: String, api_path_prefix: String) -> Self {
        Self { api_key: api_key, api_path_prefix: api_path_prefix }
    }

    pub async fn get_current(&self, city_name: &str) -> Result<WeatherReport, Box<dyn Error>> {
        let full_path = format!("{}/current?key={}&city={}", self.api_path_prefix, self.api_key, city_name);
        let raw_json = Self::get_raw(full_path).await?;
        Self::parse_report_from_raw_json(raw_json)
    }

    pub async fn get_forecast(&self, city_name: &str, days_count: usize) -> Result<Vec<WeatherReport>, Box<dyn Error>> {
        let full_path = format!("{}/forecast/daily?key={}&city={}&days={}",
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
        Self::parse_report_from_weatherbit_json_struct(&data["data"][0])
    }

    fn parse_report_array_from_raw_json(data: serde_json::Value) -> Result<Vec<WeatherReport>, Box<dyn Error>> {
        let array = data["data"].as_array();
        if array.is_some() {
            array.unwrap().into_iter().map(Self::parse_report_from_weatherbit_json_struct).collect()
        } else {
            Err(WeatherbitJsonParseError.into())
        }
    }

    fn parse_report_from_weatherbit_json_struct(data: &serde_json::Value) -> Result<WeatherReport, Box<dyn Error>> {
        let temp = data["temp"].as_f64();
        if temp.is_some() {
            Ok(WeatherReport { temperature: temp.unwrap() })
        } else {
            Err(WeatherbitJsonParseError.into())
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
            "data": [{
                "ts": 1613928600,
                "city_name": "Kazan",
                "temp": -23
            }],
            "count":1
        }
        "#;
        let json_value = serde_json::from_str(raw_json).unwrap();


        assert_eq!(
            Weatherbit::parse_report_from_raw_json(json_value).unwrap().temperature,
            -23.0
        )
    }

    #[test]
    fn it_fails_to_deserialize_current_weather_invalid_raw_json() {
        let raw_json = r#"
        {
            "data": [{
                "ts": 1613928600,
                "city_name": "Kazan",
                "tempo": -23
            }],
            "count":1
        }
        "#;
        let json_value = serde_json::from_str(raw_json).unwrap();


        assert_eq!(
            Weatherbit::parse_report_from_raw_json(json_value).is_err(),
            true
        )
    }


    #[test]
    fn it_deserializes_forecast_weather_valid_raw_json() {
        let raw_json = r#"
        {
            "data": [
                {
                    "ts": 1613941260,
                    "temp": -27.3
                },
                {
                    "ts": 1614027660,
                    "temp": -29.6
                }
            ],
            "city_name": "Kazan"
        }
        "#;
        let json_value = serde_json::from_str(raw_json).unwrap();

        let parsed_reposts =Weatherbit::parse_report_array_from_raw_json(json_value).unwrap();

        assert_eq!(parsed_reposts[0].temperature, -27.3);
        assert_eq!(parsed_reposts[1].temperature, -29.6)
    }

    #[test]
    fn it_fails_to_deserialize_forecast_weather_invalid_raw_json() {
        let raw_json = r#"
        {
            "datas": [
                {
                    "ts": 1613941260,
                    "temp": -27.3
                },
                {
                    "ts": 1614027660,
                    "temp": -29.6
                }
            ],
            "city_name": "Kazan"
        }
        "#;
        let json_value = serde_json::from_str(raw_json).unwrap();

        assert_eq!(
            Weatherbit::parse_report_array_from_raw_json(json_value).is_err(),
            true
        )
    }

    #[actix_rt::test]
    async fn it_fetches_data_from_weatherbit_service() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET)
                .path("/current");

            let json = std::fs::read_to_string("./tests/fixtures/weatherbit_current_success.json").unwrap();
            then.status(200)
                .header("Content-Type", "application/json")
                .body(json);
        });

        let key = "apikey".to_string();
        let report = Weatherbit::new_with_prefix(key, server.url("")).get_current("kazan").await;

        assert_eq!(report.is_ok(), true);
        assert_eq!(report.unwrap().temperature, -23.0);
    }

    #[actix_rt::test]
    async fn it_returns_error_for_wrong_key() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET)
                .path("/weather");

            let json = std::fs::read_to_string("./tests/fixtures/weatherbit_invalid_key.json").unwrap();
            then.status(403)
                .header("Content-Type", "application/json")
                .body(json);
        });

        let key = "apikey".to_string();
        let report = Weatherbit::new_with_prefix(key, server.url("")).get_current("kazan").await;

        assert_eq!(report.is_err(), true);
    }
}
