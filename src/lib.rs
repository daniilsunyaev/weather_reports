use actix_web::{App, HttpServer};
use actix_web::dev::Server;
use dotenv::dotenv;
use std::net::TcpListener;

mod weather_aggregator;
mod handlers;

#[derive(Debug)]
pub struct WeatherReport {
    pub temperature: f64
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    dotenv().ok();

    let server = HttpServer::new(|| {
        App::new()
            .service(handlers::daily)
            .service(handlers::forecast)
    })
    .listen(listener)?
    .run();

    Ok(server)
}
