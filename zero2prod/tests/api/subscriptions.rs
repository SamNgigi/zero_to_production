use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_sends_confirmation_email_with_confirmation_link() {
    // NOTE: ARRANGE
    let app = spawn_app().await;
    let body = "username=lei%20yin&email=lei_yin_loo%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // NOTE: ACT
    app.post_subscriptions(body.into()).await;

    // Interception the first request received by our mock email server
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    // Parse the request body into JSON
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    // Define a closure fn that extracts links given a str using linkify crate
    let get_links = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let html_body = get_links(body["HtmlBody"].as_str().unwrap());
    let text_body = get_links(body["TextBody"].as_str().unwrap());

    // NOTE: ASSERT
    assert_eq!(html_body, text_body);
}

#[tokio::test]
async fn subscribe_sends_confirmation_email_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "username=lei%20yin&email=lei_yin_loo%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1) // NOTE: -> Gets asserted at the end of the scope
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert
    // NOTE: Mock asserted on drop
}

#[tokio::test]
async fn subscribe_returns_400_when_form_fields_are_present_but_invalid() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("username=&email=lei_yin_loo%40gmail.com", "empty name"),
        ("username=lei&email=", "empty email"),
        (
            "username=lei&email=definitely-not-an-email",
            "invalid email",
        ),
    ];

    for (body, description) in test_cases {
        // Act
        let response = app.post_subscriptions(body.to_owned()).await;
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        )
    }
}

#[tokio::test]
async fn subscribe_returns_400_when_form_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("username=lei%20yin", "missing the email"),
        ("email=lei_yin_loo%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = app.post_subscriptions(invalid_body.into()).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customized error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}

#[tokio::test]
async fn subscribe_persists_new_subscriber() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let body = "username=lei%20yin&email=lei_yin_loo%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // NOTE: Act
    app.post_subscriptions(body.into()).await;

    let saved = sqlx::query!("SELECT email, username, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriber in test");

    // NOTE: Assert
    assert_eq!(saved.email, "lei_yin_loo@gmail.com");
    assert_eq!(saved.username, "lei yin");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let body = "username=lei%20yin&email=lei_yin_loo%40gmail.com";

    // Adding Mock call to email server
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // NOTE: We add the mock before posting subscriptions so that
    // a post finds our mock email_server already wired up.
    let response = app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}
