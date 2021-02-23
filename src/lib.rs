use actix_web::{App, HttpServer};
use actix_web::dev::Server;
use dotenv::dotenv;
use std::net::TcpListener;
use std::env;

mod weather_aggregator;
mod handlers;

#[derive(Debug)]
pub struct WeatherReport {
    pub temperature: f64,
    pub unix_timestamp: i64
}

impl WeatherReport {
    pub fn mean_merge(&self, another_report: WeatherReport) -> WeatherReport {
        WeatherReport {
            temperature: (self.temperature + another_report.temperature) / 2.0,
            unix_timestamp: (self.unix_timestamp + another_report.unix_timestamp) / 2
        }
    }
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    dotenv().ok();
    verify_env_vars();

    let server = HttpServer::new(|| {
        App::new()
            .service(handlers::daily)
            .service(handlers::forecast)
    })
    .listen(listener)?
    .run();

    Ok(server)
}

fn verify_env_vars() {
    env::var("OPEN_WEATHER_APPID").expect("OPEN_WEATHER_APPID is not specified");
    env::var("WEATHERBIT_API_KEY").expect("WEATHERBIT_API_KEY is not specified");
}
