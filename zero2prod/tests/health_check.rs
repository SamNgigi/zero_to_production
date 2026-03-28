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
    spawn_app().await;
    // Perfoming HTTP requests against our application using reqwest
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get("http://127.0.0.1:3000/health_check")
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

// Launching our application in the background ~somehow~
async fn spawn_app() {
    let server = zero2prod::run().expect("Failed to bind address");

    let _task = tokio::spawn(server);
}
