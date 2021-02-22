use std::net::TcpListener;

#[actix_rt::test]
async fn get_current_weather() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/daily?city_name=london", &address))
        .send()
        .await
        .expect("Failed to execute request");

    println!("{:?}", response);
    assert!(response.status().is_success());
}

#[actix_rt::test]
async fn get_forecast_weather() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/forecast?city_name=paris", &address))
        .send()
        .await
        .expect("Failed to execute request");

    println!("{:?}", response);
    assert!(response.status().is_success());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = weather_reports::run(listener).expect("Failed to launch the app");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
