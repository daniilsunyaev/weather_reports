use actix_web::{web, get, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use crate::WeatherReport;

#[derive(Deserialize)]
pub struct DailyParams {
    city_name: Option<String>,
    days_since: Option<usize>
}


#[get("/daily")]
async fn daily(web::Query(params): web::Query<DailyParams>) -> impl Responder {
    if params.city_name.is_none() {
        HttpResponse::UnprocessableEntity().body("city_name should be specified")
            // TODO: switch days to string and check positive here
    } else {
        let report : Result<WeatherReport, String>;
        if params.days_since.is_none() {
            report = crate::get_current_weather(params.city_name.unwrap().as_str()).await;
        } else {
            report = crate::get_specific_day_weather(params.city_name.unwrap().as_str(), params.days_since.unwrap()).await;
        };

        if report.is_ok() {
            HttpResponse::Ok().body(format!("Temperature: {}", report.unwrap().temperature))
        } else {
            HttpResponse::NotFound().body(report.err().unwrap())
        }
    }
}
