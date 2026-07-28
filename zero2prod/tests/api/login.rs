use crate::helpers::spawn_app;

#[tokio::test]
async fn redirects_to_admin_dashboard_on_successful_login() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let login_request = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.username,
    });

    // NOTE: Act & Assert 1
    let response = app.post_login(&login_request).await;
    assert_on_redirect(&response, "/admin_dashboard");

    // NOTE: Act & Assert 1
    let admin_dashboard_html = app.get_admin_dashboard_html().await;
    assert!(admin_dashboard_html.contains(&format!("Welcome {}", app.test_user.username)));
}

#[tokio::test]
async fn an_error_flash_message_cookie_is_set_on_failure() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password",
    });

    // NOTE: Act and Assert 1
    let response = app.post_login(&login_body).await;
    assert_on_redirect(&response, "/login");

    // NOTE: Act and Assert 2
    let login_html = app.get_login_html().await;
    assert!(
        login_html.contains(r#"<p><i>Authentication Failed.</i></p>"#),
        "Error Html Should Be Rendered."
    );

    // NOTE: Act and Assert 2
    let login_html = app.get_login_html().await;
    assert!(
        !login_html.contains(r#"<p><i>Authentication Failed.</i></p>"#),
        "Error Html Should NOT Be Rendered."
    );
}

fn assert_on_redirect(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(
        response
            .headers()
            .get("Location")
            .expect("Failed to get location header."),
        location
    );
}
