use actix_web::{web, get, HttpResponse, Responder};
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
            HttpResponse::Ok().body(format_daily_report(report.unwrap()))
        } else {
            HttpResponse::NotFound().body(report.err().unwrap())
        }
    }
}

#[derive(Deserialize)]
pub struct ForecastParams {
    city_name: Option<String>,
}

#[get("/forecast")]
async fn forecast(web::Query(params): web::Query<ForecastParams>) -> impl Responder {
    if params.city_name.is_none() {
        HttpResponse::UnprocessableEntity().body("city_name should be specified")
    } else {
        let report = crate::get_forecast_weather(params.city_name.unwrap().as_str(), 5).await;

        if report.is_ok() {
            HttpResponse::Ok().body(format_forecat_report(report.unwrap()))
        } else {
            HttpResponse::NotFound().body(report.err().unwrap())
        }
    }
}

fn format_forecat_report(reports: Vec<WeatherReport>) -> String {
    let mut result_as_string = String::from("");
    let mut date = chrono::offset::Local::today();
    for i in 0..reports.len() {
        result_as_string = format!("{}{}, temperature: {}\n",
                                   result_as_string, date.format("%a %b %e").to_string(), reports[i].temperature);
        date = date.succ();
    }
    result_as_string
}

fn format_daily_report(report: WeatherReport) -> String {
    let date = chrono::offset::Local::today();
    format!("{}, temperature: {}\n", date.format("%a %b %e").to_string(), report.temperature)
}
