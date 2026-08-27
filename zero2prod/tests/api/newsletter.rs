use crate::helpers::{ConfirmationLinks, TestApp, assert_on_redirect, spawn_app};
use uuid::Uuid;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{any, method, path},
};

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    // NOTE: Arrange
    let app = spawn_app().await;
    app.test_user.login(&app).await;
    create_confirmed_subscriber(&app).await;
    let post_newsletter_request = serde_json::json!({
        "title": "Newsletter title",
        "txt_content": "Newsletter content",
        "idempotency_key": Uuid::now_v7().to_string(),
    });

    // NOTE: Arrange
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1) // NOTE: -> asserted on drop.
        .mount(&app.email_server)
        .await;

    // NOTE: Assert 1: 1st Newsletter published successfully & flash message rendered
    let response = app.post_publish_newsletter(&post_newsletter_request).await;
    assert_on_redirect(&response, "/admin/publish_newsletter");
    let publish_newsletter_html = app.get_publish_newsletter_html().await;
    assert!(
        publish_newsletter_html
            .contains(r#"<p><i>Newsletter Issue Published Successfully.</i></p>"#)
    );

    // NOTE: Assert 2: Retry successful (email not resent)
    let response = app.post_publish_newsletter(&post_newsletter_request).await;
    assert_on_redirect(&response, "/admin/publish_newsletter");
    let publish_newsletter_html = app.get_publish_newsletter_html().await;
    assert!(
        publish_newsletter_html
            .contains(r#"<p><i>Newsletter Issue Published Successfully.</i></p>"#)
    );

    // NOTE: Assert 3: Mock asserts on Drop that newsletter email was sent only once
}

#[tokio::test]
async fn publish_newsletter_works() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act + Assert 1 - Successful login
    let login_request = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password,
    });
    let response = app.post_login(&login_request).await;
    assert_on_redirect(&response, "/admin/dashboard");
    let admin_dashboard_html = app.get_admin_dashboard_html().await;
    assert!(admin_dashboard_html.contains(&format!("<p>Welcome {}.</p>", app.test_user.username)));

    // NOTE: Act + Assert 2 - Redirect to GET /admin/publish_newsletter
    let response = app
        .post_publish_newsletter(&serde_json::json!({
            "title": "Newsletter title",
            "txt_content": "Newsletter content.",
            "idempotency_key": Uuid::now_v7().to_string(),
        }))
        .await;
    assert_on_redirect(&response, "/admin/publish_newsletter");

    // NOTE: Act + Assert 3 - Publish newsletter successful flash message is rendered
    let publish_newsletter_html = app.get_publish_newsletter_html().await;
    assert!(
        publish_newsletter_html
            .contains(r#"<p><i>Newsletter Issue Published Successfully.</i></p>"#)
    );
}

#[tokio::test]
async fn error_flash_message_is_set_on_missing_content_for_newsletter_issue() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act + Assert 1 - Successful login
    let login_request = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password,
    });
    let response = app.post_login(&login_request).await;
    assert_on_redirect(&response, "/admin/dashboard");
    let admin_dashboard_html = app.get_admin_dashboard_html().await;
    assert!(admin_dashboard_html.contains(&format!("<p>Welcome {}.</p>", app.test_user.username)));

    // NOTE: Act + Assert 2 - Redirect to GET /admin/publish_newsletter on missing content
    let response = app
        .post_publish_newsletter(&serde_json::json!({
            "title": "Newsletter title",
            "txt_content": "",
            "idempotency_key": Uuid::now_v7(),
        }))
        .await;
    assert_on_redirect(&response, "/admin/publish_newsletter");

    // NOTE: Act + Assert 3 - Flash Error message is rendered
    let publish_newsletter_html = app.get_publish_newsletter_html().await;
    assert!(publish_newsletter_html.contains(
        r#"<p><i>Newsletter issue is missing content. Issue must have a content.</i></p>"#
    ));
}

#[tokio::test]
async fn error_flash_message_is_set_on_missing_title_for_newsletter_issue() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act + Assert 1 - Succesful login
    let login_request = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password,
    });
    let response = app.post_login(&login_request).await;
    assert_on_redirect(&response, "/admin/dashboard");
    let admin_dashboard_html = app.get_admin_dashboard_html().await;
    assert!(admin_dashboard_html.contains(&format!("<p>Welcome {}.</p>", app.test_user.username)));

    // NOTE: Act + Assert 2 - Redirect to GET /admin/publish_newsletter on missing title
    let response = app
        .post_publish_newsletter(&serde_json::json!({
            "title": "",
            "txt_content": "Newsletter content",
            "idempotency_key": Uuid::now_v7(),
        }))
        .await;
    assert_on_redirect(&response, "/admin/publish_newsletter");

    // NOTE: Act + Assert 3 - Flash Error message is rendered
    let publish_newsletter_html = app.get_publish_newsletter_html().await;
    assert!(publish_newsletter_html.contains(
        r#"<p><i>Newsletter issue is missing a title. Issue must have a title.</i></p>"#
    ));
}

#[tokio::test]
async fn you_must_be_logged_in_to_post_to_publish_newsletter() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act
    let response = app
        .post_publish_newsletter(&serde_json::json!({
            "title": "Newsletter title",
            "txt_content": "Newsletter content.",
            "idempotency_key": Uuid::now_v7().to_string(),
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

    // NOTE: Act
    let response = app
        .post_publish_newsletter(&serde_json::json!({
            "title": "Newsletter title",
            "txt_content": "Newsletter content.",
            "idempotency_key": Uuid::now_v7().to_string(),
        }))
        .await;

    // NOTE: Assert
    assert_eq!(response.status().as_u16(), 303);
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

    let response = app
        .post_publish_newsletter(&serde_json::json!({
            "title": "Newsletter title",
            "txt_content": "Newsletter content.",
            "idempotency_key": Uuid::now_v7().to_string(),
        }))
        .await;
    assert_eq!(response.status().as_u16(), 303);
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
