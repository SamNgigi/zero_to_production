use crate::common::spawn_app;

use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

#[tokio::test]
async fn confirmation_link_returned_by_subscribe_returns_200_when_called() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let body = "username=lei%20yin&email=lei_yin_loo%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    // Intercept email request
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    // Get confirmation links
    let confirmation_links = app.get_confirmation_links(email_request);

    // NOTE: Act
    let response = reqwest::get(confirmation_links.html)
        .await
        .expect("Failed to execute confirmation_link request");
    dbg!(&response);

    // NOTE: Assert
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn confirmation_link_without_subscription_token_rejected_with_400() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act
    let response = reqwest::get(format!("{}/subscriptions/confirm", app.address))
        .await
        .expect("Failed to execute get subscriptions confirm request");

    // NOTE: Assert
    assert_eq!(response.status().as_u16(), 400);
}
