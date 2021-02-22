use actix_web::{web, get, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use dotenv::dotenv;

#[derive(Deserialize)]
pub struct DailyParams {
    city_name: Option<String>,
    days_since: Option<usize>
}

#[get("/daily")]
async fn daily(web::Query(params): web::Query<DailyParams>) -> impl Responder {
    if params.city_name.is_none() {
        HttpResponse::UnprocessableEntity().body("city_name should be specified")
    } else {
        let report : Result<weather_reports::WeatherReport, String>;
        if params.days_since.is_none() {
            report = weather_reports::get_current_weather(params.city_name.unwrap().as_str()).await;
        } else {
            report = weather_reports::get_specific_day_weather(params.city_name.unwrap().as_str(), params.days_since.unwrap()).await;
        };

        if report.is_ok() {
            HttpResponse::Ok().body(format!("Temperature: {}", report.unwrap().temperature))
        } else {
            HttpResponse::NotFound().body(report.err().unwrap())
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    HttpServer::new(|| {
        App::new()
            .service(daily)
    })
    .bind("127.0.0.1:7878")?
    .run()
    .await
}
