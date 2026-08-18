use secrecy::ExposeSecret;

use crate::common::{assert_on_redirect, spawn_app};

#[tokio::test]
async fn logout_clears_session_state() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act & Assert 1: Successful login
    let response = app
        .post_login(&serde_json::json!({
            "username": app.test_user.username,
            "password": app.test_user.password.expose_secret(),
        }))
        .await;
    assert_on_redirect(&response, "/admin/dashboard");
    // Following redirect and check dashboard for username
    let dashboard_html = app.get_admin_dashboard_html().await;
    assert!(dashboard_html.contains(&format!("Welcome {}.", app.test_user.username)));

    // NOTE: Act & Assert: Successful logout.
    let response = app.post_logout().await;
    assert_on_redirect(&response, "/login");
    // Follow redirect and check logout has appropriate flash message.
    let login_html = app.get_login_html().await;
    assert!(login_html.contains(r#"<p><i>You've been successfully logged out.</i><p>"#))
}

#[tokio::test]
async fn you_must_be_logged_in_to_access_admin_dashboard() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act
    let response = app.get_admin_dashboard().await;

    // NOTE: Assert
    assert_on_redirect(&response, "/login");
}
