use crate::helpers::{ConfirmationLinks, TestApp, assert_on_redirect, spawn_app};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{any, method, path},
};

#[tokio::test]
async fn you_must_be_logged_in_to_post_to_publish_newsletter() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act
    let response = app
        .post_publish_newsletter(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "plain": "Newsletter as plain text",
                "html": "<p>Newsletter as HTML</p>"
            }
        }))
        .await;

    // NOTE: Assert
    assert_on_redirect(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_access_publish_newsletter_form() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act
    let response = app.get_publish_newsletter().await;

    // NOTE: Assert
    assert_on_redirect(&response, "/login");
}

#[tokio::test]
async fn newsletters_return_400_for_invalid_data() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!({
                "content": {
                    "plain": "Newsletter as plain text",
                    "html": "<p>Newsletter as plain text</p>",
                }
            }),
            "Missing title",
        ),
        (
            serde_json::json!({"title": "Newsletter title!"}),
            "Missing content",
        ),
    ];

    // NOTE: Act
    for (invalid_data, error_msg) in test_cases {
        let response = app.post_newsletters(invalid_data).await;

        // NOTE: Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when payload was: {}",
            error_msg
        )
    }
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act & Assert 1 - Succesful Login
    let login_request = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password,
    });
    let response = app.post_login(&login_request).await;
    assert_on_redirect(&response, "/admin/dashboard");
    let admin_dashboard_html = app.get_admin_dashboard_html().await;
    assert!(admin_dashboard_html.contains(&format!("<p>Welcome {}.</p>", app.test_user.username)));

    // NOTE: Act & Assert 2 - Newsletters are delivered to confirmed subs
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "plain": "Newsletter as plain text",
            "html": "<p>Newsletter as HTML</p>",
        }
    });

    // NOTE: Act
    let response = app.post_publish_newsletter(&newsletter_request_body).await;

    // NOTE: Assert
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act & Assert 1 - Succesful Login
    let login_request = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password,
    });
    let response = app.post_login(&login_request).await;
    assert_on_redirect(&response, "/admin/dashboard");
    let admin_dashboard_html = app.get_admin_dashboard_html().await;
    assert!(admin_dashboard_html.contains(&format!("<p>Welcome {}.</p>", app.test_user.username)));

    // NOTE: Act & Assert 2 - Newsletters are not delivered to unconfirmed subs
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title.",
        "content": {
            "plain": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });

    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    dbg!(&response);
    assert_eq!(response.status().as_u16(), 200);
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_links = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "username=lei%yin&email=lei_yin_loo%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Mock email server for unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .expect("Failed to create unconfirmed subscriber in test");

    let request_body = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_links(request_body)
}
