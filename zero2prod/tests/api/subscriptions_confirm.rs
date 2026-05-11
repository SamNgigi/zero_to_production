use crate::helpers::spawn_app;

use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

#[tokio::test]
async fn clicked_confirmation_link_confirms_new_subscriber() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let body = "username=lei%20yin&email=lei_yin_loo%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let received_requests = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(received_requests);

    // NOTE: Act
    reqwest::get(confirmation_links.html)
        .await
        .expect("Failed to execute confirmation request in test");

    let saved = sqlx::query!("SELECT email, username, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to execute confirmed user query in test");

    // NOTE: Assert
    assert_eq!(saved.email, "lei_yin_loo@gmail.com");
    assert_eq!(saved.username, "lei yin");
    assert_eq!(saved.status, "confirmed");
}

#[tokio::test]
async fn confirmation_link_returns_200_when_called() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let body = "username=lei%20yin&email=lei_yin_loo%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    // Intercepting the first email request
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    // NOTE: Act
    let response = reqwest::get(confirmation_links.html)
        .await
        .expect("Failed to execute confirmation request in test");

    // NOTE: Assert
    assert_eq!(response.status().as_u16(), 200);
}

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
