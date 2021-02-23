use actix_web::{web, get, HttpResponse, Responder};
use serde::Deserialize;
use crate::WeatherReport;
use crate::weather_aggregator;
use chrono::NaiveDateTime;

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
            report = weather_aggregator::get_current_weather(params.city_name.unwrap().as_str()).await;
        } else {
            report = weather_aggregator::get_specific_day_weather(params.city_name.unwrap().as_str(), days_since.unwrap()).await;
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
        let report = weather_aggregator::get_forecast_weather(params.city_name.unwrap().as_str(), 5).await;

        if report.is_ok() {
            HttpResponse::Ok().body(format_forecast_report(report.unwrap()))
        } else {
            HttpResponse::NotFound().body(report.err().unwrap())
        }
    }
}

fn format_forecast_report(reports: Vec<WeatherReport>) -> String {
    let mut result_as_string = String::from("");
    for report in reports {
        result_as_string = format!("{}{}\n", result_as_string, format_daily_report(report))
    }
    result_as_string
}

fn format_daily_report(report: WeatherReport) -> String {
    let date = NaiveDateTime::from_timestamp(report.unix_timestamp, 0);
    format!("{}, temperature: {}", date.format("%a %b %e").to_string(), report.temperature)
}
