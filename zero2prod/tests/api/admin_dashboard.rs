use crate::helpers::{assert_on_redirect, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_access_admin_dashboard() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act
    let response = app.get_admin_dashboard().await;

    // NOTE: Arrange
    assert_on_redirect(&response, "/login");
}

#[tokio::test]
async fn logout_clears_session_state() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act & Assert 1 - Successful login
    let login_request = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password
    });
    let response = app.post_login(&login_request).await;
    assert_on_redirect(&response, "/admin/dashboard");
    let admin_dashboard_html = app.get_admin_dashboard_html().await;
    assert!(admin_dashboard_html.contains(&format!("<p>Welcome {}.</p>", app.test_user.username)));

    // NOTE: Act & Assert 2 - Successful logout.
    let response = app.post_logout().await;
    assert_on_redirect(&response, "/login");
    let login_html = app.get_login_html().await;
    assert!(login_html.contains(r#"<p><i>You've successfully logged out.</i></p>"#));

    // NOTE: Act & Assert 3 - Cannot access dashboard.
    let response = app.get_admin_dashboard().await;
    assert_on_redirect(&response, "/login");
}
