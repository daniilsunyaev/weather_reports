#[actix_rt::test]
async fn get_current_weather() {
    spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get("http://localhost:7878/daily?city_name=london")
        .send()
        .await
        .expect("Failed to execute request");

    println!("{:?}", response);
    assert!(response.status().is_success());
}

fn spawn_app() {
    let server = weather_reports::run().expect("Failed to launch the app");
    let _ = tokio::spawn(server);
}
