use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;

mod weather_clients;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hey, world")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    HttpServer::new(|| {
        App::new()
            .service(hello)
    })
    .bind("127.0.0.1:7878")?
    .run()
    .await
}
