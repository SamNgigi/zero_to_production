use fake::{
    Fake,
    faker::internet::en::{SafeEmail, Username},
};
use secrecy::ExposeSecret;
use std::time::Duration;
use uuid::Uuid;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{any, method, path},
};

use crate::common::{ConfirmationLinks, TestApp, assert_on_redirect, spawn_app};

#[tokio::test]
async fn transient_errors_do_not_cause_duplicate_deliveries_on_retry() {
    // NOTE: Arrange
    let app = spawn_app().await;
    app.test_user.login(&app).await;
    let publish_newsletter_request = serde_json::json!({
        "title": "Newsletter title",
        "txt_content": "Newsletter content.",
        "idempotency_key": Uuid::now_v7().to_string(),
    });

    // NOTE: We create 2 confirmed subscribers
    // We updated implementation to use fake to generate
    // 2 distinct subscribers
    create_confirmed_subscriber(&app).await;
    create_confirmed_subscriber(&app).await;

    // NOTE: Act & Assert 1 - First email send for first confirmed sub.
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;

    // NOTE: Act & Assert 2 - Second email send for second confirmed sub.
    // We intentonally cause a transient error with a 500 response
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app
        .post_publish_newsletter(&publish_newsletter_request)
        .await;
    assert_eq!(response.status().as_u16(), 500);

    // NOTE: Act & Assert 3 - A retry that shouldn't send the intial successful delivery
    // just the second one that failed.
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .named("Delivery Retry")
        .mount(&app.email_server)
        .await;

    let response = app
        .post_publish_newsletter(&publish_newsletter_request)
        .await;
    assert_eq!(response.status().as_u16(), 303);
}

#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() {
    // NOTE: Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;
    let publish_newsletter_request = serde_json::json!({
        "title": "Newsletter title",
        "txt_content": "Newsletter content.",
        "idempotency_key": Uuid::now_v7().to_string(),
    });

    // NOTE: Act
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1) // NOTE: -> This is asserted on drop
        .named("Concurrent Delivery Retry")
        .mount(&app.email_server)
        .await;

    let response1 = app.post_publish_newsletter(&publish_newsletter_request);
    let response2 = app.post_publish_newsletter(&publish_newsletter_request);
    let (response1, response2) = tokio::join!(response1, response2);

    // NOTE: Assert 1 - We get equivalent values from responses.
    assert_eq!(response1.status().as_u16(), response2.status().as_u16());
    assert_eq!(
        response1.text().await.unwrap(),
        response2.text().await.unwrap()
    );

    // NOTE: Assert 2 - Mock asserted on drop that we only received on request to the email server.
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    // NOTE: Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;
    let publish_newsletter_request = serde_json::json!({
        "title": "Newsletter title",
        "txt_content": "Newsletter content.",
        "idempotency_key": Uuid::now_v7().to_string(),
    });

    // NOTE: Act
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1) // NOTE: Expecting only one email send request if idempotent. Asserted on drop
        .mount(&app.email_server)
        .await;

    // NOTE: Assert 1 - newsletter published successfully.
    let response = app
        .post_publish_newsletter(&publish_newsletter_request)
        .await;
    assert_on_redirect(&response, "/admin/publish_newsletter");
    let publish_newsletter_html = app.get_publish_newsletter_html().await;
    assert!(
        publish_newsletter_html
            .contains(r#"<p><i>Newsletter Issue Published Successfully.</i></p>"#)
    );
    // NOTE: The retry.
    let response = app
        .post_publish_newsletter(&publish_newsletter_request)
        .await;
    assert_on_redirect(&response, "/admin/publish_newsletter");
    let publish_newsletter_html = app.get_publish_newsletter_html().await;
    assert!(
        publish_newsletter_html
            .contains(r#"<p><i>Newsletter Issue Published Successfully.</i></p>"#)
    );

    // NOTE: Mock asserted on drop. We expect only one request to hit the email server, if indempotent
}

#[tokio::test]
async fn publish_newsletter_issue_works() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let login_request = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password.expose_secret(),
    });

    // NOTE: Act & Assert 1 - Successful login
    let response = app.post_login(&login_request).await;
    assert_on_redirect(&response, "/admin/dashboard");
    // Following redirect and checking username on admin dashboard
    let admin_dashboard_html = app.get_admin_dashboard_html().await;
    assert!(admin_dashboard_html.contains(&format!("Welcome {}.", app.test_user.username)));

    // NOTE: Act & Assert 1 - Missing content flash error
    let publish_newsletter_request = serde_json::json!({
        "title": "Newsletter issue title",
        "txt_content": "Newsletter issue content.",
        "idempotency_key": Uuid::now_v7().to_string(),
    });
    let response = app
        .post_publish_newsletter(&publish_newsletter_request)
        .await;
    assert_on_redirect(&response, "/admin/publish_newsletter");
    // Following redirect and check flash error message is rendered.
    let publish_newsletter_html = app.get_publish_newsletter_html().await;
    assert!(
        publish_newsletter_html
            .contains(r#"<p><i>Newsletter Issue Published Successfully.</i></p>"#)
    )
}

#[tokio::test]
async fn error_flash_message_is_set_on_missing_newsletter_issue_content() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let login_request = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password.expose_secret(),
    });

    // NOTE: Act & Assert 1 - Successful login
    let response = app.post_login(&login_request).await;
    assert_on_redirect(&response, "/admin/dashboard");
    // Following redirect and checking username on admin dashboard
    let admin_dashboard_html = app.get_admin_dashboard_html().await;
    assert!(admin_dashboard_html.contains(&format!("Welcome {}.", app.test_user.username)));

    // NOTE: Act & Assert 1 - Missing content flash error
    let publish_newsletter_request = serde_json::json!({
        "title": "Newsletter issue title",
        "txt_content": "",
    });
    let response = app
        .post_publish_newsletter(&publish_newsletter_request)
        .await;
    assert_on_redirect(&response, "/admin/publish_newsletter");
    // Following redirect and check flash error message is rendered.
    let publish_newsletter_html = app.get_publish_newsletter_html().await;
    assert!(publish_newsletter_html.contains(
        r#"<p><i>Missing content for newsletter issue. Issue must have content.</i></p>"#
    ))
}

#[tokio::test]
async fn error_flash_message_is_set_on_missing_newsletter_issue_title() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let login_request = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password.expose_secret(),
    });

    // NOTE: Act & Assert 1 - Successful login
    let response = app.post_login(&login_request).await;
    assert_on_redirect(&response, "/admin/dashboard");
    // Following redirect and checking username on admin dashboard
    let admin_dashboard_html = app.get_admin_dashboard_html().await;
    assert!(admin_dashboard_html.contains(&format!("Welcome {}.", app.test_user.username)));

    // NOTE: Act & Assert 1 - Missing title flash error
    let publish_newsletter_request = serde_json::json!({
        "title": "",
        "txt_content": "Newsletter issue content.",
    });
    let response = app
        .post_publish_newsletter(&publish_newsletter_request)
        .await;
    assert_on_redirect(&response, "/admin/publish_newsletter");
    // Following redirect and check flash error message is rendered.
    let publish_newsletter_html = app.get_publish_newsletter_html().await;
    assert!(
        publish_newsletter_html.contains(
            r#"<p><i>Missing title for newsletter issue. Issue must have a title.</i></p>"#
        )
    )
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let login_request = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password.expose_secret(),
    });

    // NOTE: Act & Assert 1 - Successful login
    let response = app.post_login(&login_request).await;
    assert_on_redirect(&response, "/admin/dashboard");
    // Following redirect and checking username on admin dashboard
    let admin_dashboard_html = app.get_admin_dashboard_html().await;
    assert!(admin_dashboard_html.contains(&format!("Welcome {}.", app.test_user.username)));

    // NOTE: Act & Assert 2 - Newsletter are delivered
    create_confirmed_subscriber(&app).await;
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // NOTE: Act
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title!",
        "txt_content": "Newsletter issue content",
        "idempotency_key": Uuid::now_v7().to_string(),
    });

    let response = app.post_publish_newsletter(&newsletter_request_body).await;

    // NOTE: Assert
    assert_on_redirect(&response, "/admin/publish_newsletter")
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let login_request = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password.expose_secret(),
    });

    // NOTE: Act & Assert 1 - Successful login
    let response = app.post_login(&login_request).await;
    assert_on_redirect(&response, "/admin/dashboard");
    // Following redirect and checking username on admin dashboard
    let admin_dashboard_html = app.get_admin_dashboard_html().await;
    assert!(admin_dashboard_html.contains(&format!("Welcome {}.", app.test_user.username)));

    // NOTE: Act & Assert 2 - Newsletter not delivered
    create_unconfirmed_subscriber(&app).await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0) // email server should not receive any request. Asserted at end of scope
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title!",
        "txt_content": "Newsletter issue content",
        "idempotency_key": Uuid::now_v7().to_string(),
    });

    let response = app.post_publish_newsletter(&newsletter_request_body).await;

    // NOTE: Arrange
    assert_on_redirect(&response, "/admin/publish_newsletter")
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_links = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .expect("Failed to execute request to create confirmed subscriber in test");
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let username: String = Username().fake();
    let email: String = SafeEmail().fake();
    let body = serde_urlencoded::to_string(serde_json::json!({
        "username": username,
        "email": email
    }))
    .unwrap();
    // let body = "username=lei%20yin&email=lei_yin_loo%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create Unconfirmed Subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body)
        .await
        .error_for_status()
        .expect("Failed to create unconfirmed subscriber in test");

    let received_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_links(received_request)
}

#[tokio::test]
async fn you_must_be_logged_in_to_post_to_publish_newsletter() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act
    let response = app
        .post_publish_newsletter(&serde_json::json!({
            "title": "Newsletter title",
            "txt_content": "Newsletter issue content."
        }))
        .await;

    // NOTE: Assert
    assert_on_redirect(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_access_publish_newsletter() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act
    let response = app.get_publish_newsletter().await;

    // NOTE: Assert
    assert_on_redirect(&response, "/login");
}
