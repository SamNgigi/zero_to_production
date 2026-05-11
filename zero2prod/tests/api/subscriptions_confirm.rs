use crate::helpers::spawn_app;

use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

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
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    // Closure to extract link from email body
    let get_links = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let raw_confirmation_link = &get_links(&body["HtmlBody"].as_str().unwrap());
    let confirmation_link = reqwest::Url::parse(raw_confirmation_link).unwrap();
    // Confirming that we don't call random APIs on the web
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

    // NOTE: Act
    let response = reqwest::get(confirmation_link)
        .await
        .expect("Failed to execute confirmation request in test");

    // NOTE: Assert
    assert_eq!(response.status().as_u16(), 200);
}
