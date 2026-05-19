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
    // Parse into json
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    // Extract confirmation_link
    let raw_confirmation_link = get_link(body["HtmlBody"].as_str().unwrap());
    let mut confirmation_link = reqwest::Url::parse(&raw_confirmation_link).unwrap();
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
    confirmation_link.set_port(Some(app.port)).unwrap();

    // NOTE: Act
    dbg!(&confirmation_link);
    let response = reqwest::get(confirmation_link)
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
