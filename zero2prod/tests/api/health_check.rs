use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let address = format!("{}/health_check", &app.address);
    let response = reqwest::Client::new()
        .get(address)
        .send()
        .await
        .expect("Failed to execute health_check request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
