use crate::common::spawn_app;

use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

#[tokio::test]
async fn subscribe_sends_confirmation_email_for_valid_data() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let body = "username=lei%20yin&email=lei_yin_loo%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // NOTE: Act
    app.post_subscriptions(body.into()).await;

    // NOTE:
    // Assertion that we received our mock server request
    // happens at the end of scope
}

#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_invalid() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = [
        ("username=&email=lei_yin_loo%40gmail.com", "empty name"),
        ("username=lei&email=", "empty email"),
        (
            "username=lei&email=definitely-not-an-email",
            "invalid email",
        ),
    ];

    // Act
    for (invalid_body, description) in test_cases {
        let response = app.post_subscriptions(invalid_body.to_string()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when payload was {}",
            description,
        )
    }
}

#[tokio::test]
async fn subscribe_returns_422_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = [
        ("username=lei%20yin", "missing the email"),
        ("email=lei_yin_loo%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    // Act
    for (body, description) in test_cases {
        let response = app.post_subscriptions(body.to_string()).await;

        // Assert
        assert_eq!(
            // Axum returns HTTP-semantically correct choice
            // - 422 UNPROCESSABLE ENTITY (I understood the content type but body is invalid)
            422,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when payload was {}",
            description,
        )
    }
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "username=lei%20yin&email=lei_yin_loo%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Act
    let response = app.post_subscriptions(body.to_string()).await;
    // Assert
    assert_eq!(200, response.status().as_u16());

    // Act
    let result = sqlx::query!("SELECT username, email FROM subscriptions;")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to execute db query in test");

    // Assert
    assert_eq!(result.username, "lei yin");
    assert_eq!(result.email, "lei_yin_loo@gmail.com");
}
