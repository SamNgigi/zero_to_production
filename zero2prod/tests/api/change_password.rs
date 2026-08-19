use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::common::{assert_on_redirect, spawn_app};

/// NOTE: `new_password` and `confirm_password` should match
#[tokio::test]
async fn error_flash_message_is_set_on_new_password_fields_mismatch() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let login_request = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password.expose_secret(),
    });
    let change_password_request = serde_json::json!({
        "current_password": app.test_user.username,
        "new_password": Uuid::new_v4().to_string(),
        "confirm_password": Uuid::new_v4().to_string(),
    });

    // NOTE: Act & Assert 1 - Successful login
    let response = app.post_login(&login_request).await;
    assert_on_redirect(&response, "/admin/dashboard");
    // Following redirect and checking username on admin dashboard
    let admin_dashboard_html = app.get_admin_dashboard_html().await;
    assert!(admin_dashboard_html.contains(&format!("Welcome {}.", app.test_user.username)));

    // NOTE: Act & Assert 2 - Flash message for new password field mismatch
    let response = app.post_change_password(&change_password_request).await;
    assert_on_redirect(&response, "/admin/change_password");
    // Following redirect and checking error message is rendered.
    let change_password_html = app.get_change_password_html().await;
    assert!(change_password_html.contains(
        r#"<p><i>New password and Confirm Password field DO NOT match. Fields must match.</i></p>"#
    ))
}

#[tokio::test]
async fn you_must_be_logged_in_to_post_to_change_password() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    // NOTE: Act
    let change_password_request = serde_json::json!({
        "current_password": Uuid::new_v4().to_string(),
        "new_password": &new_password,
        "confirm_password": &new_password
    });
    let response = app.post_change_password(&change_password_request).await;

    // NOTE: Assert
    assert_on_redirect(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_access_change_password() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act &
    let response = app.get_change_password().await;

    // NOTE: Assert
    assert_on_redirect(&response, "/login");
}
