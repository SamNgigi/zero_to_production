use crate::helpers::spawn_app;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_400() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act
    let response = reqwest::Client::new()
        .get(format!("{}/subscriptions/confirm", app.address))
        .send()
        .await
        .expect("Failed to execute confrimation get request in test");

    assert_eq!(response.status().as_u16(), 400);
}
