use actix_web::{web, get, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use crate::WeatherReport;

#[derive(Deserialize)]
pub struct DailyParams {
    city_name: Option<String>,
    days_since: Option<String>
}


#[get("/daily")]
async fn daily(web::Query(params): web::Query<DailyParams>) -> impl Responder {
    let mut days_since: Option<usize> = None;
    if let Some(days_since_str) = &params.days_since {
        days_since = days_since_str.parse().ok();
    };
    if params.city_name.is_none() {
        HttpResponse::UnprocessableEntity().body("city_name should be specified")
    } else if params.days_since.is_some() && days_since.is_none() {
        HttpResponse::UnprocessableEntity().body("days_since should be non-negative number")
    } else {
        let report : Result<WeatherReport, String>;
        if days_since.is_none() {
            report = crate::get_current_weather(params.city_name.unwrap().as_str()).await;
        } else {
            report = crate::get_specific_day_weather(params.city_name.unwrap().as_str(), days_since.unwrap()).await;
        };

        if report.is_ok() {
            HttpResponse::Ok().body(format!("Temperature: {}", report.unwrap().temperature))
        } else {
            HttpResponse::NotFound().body(report.err().unwrap())
        }
    }
}
