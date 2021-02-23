use weather_reports::run;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Failed to bind port 7878");
    run(listener)?.await
}
