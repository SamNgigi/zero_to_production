use std::net::TcpListener;

/*
* `tokio::test` is the testing equivalent of `tokio::main`.
* It also spares us from having to specify the `#[test]` attribute.
*
* We can inspect what code gets generated using
* `cargo expand --test health_chect` (<- name of the test file)
* */
#[tokio::test]
async fn test_heath_check() {
    // Arrange
    let address = spawn_app().await;
    // Perfoming HTTP requests against our application using reqwest
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

// Launching our application in the background ~somehow~
async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    // Retrieving hte port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Failed to bind address");
    let _task = tokio::spawn(server);

    // Returning the application address to the caller
    format!("http://127.0.0.1:{}", port)
}
