use tokio::net::TcpListener as TokioTcpListener;

#[tokio::test]
async fn test_health_check() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute requests.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

async fn spawn_app() -> String {
    let listener = TokioTcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");

    // Retrieving the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        zero2prod::run(listener)
            .await
            .expect("Failed to run app in test");
    });

    format!("http://127.0.0.1:{}", port)
}
