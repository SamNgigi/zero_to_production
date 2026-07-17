use crate::helpers::spawn_app;

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

#[tokio::test]
async fn an_error_flash_message_cookie_is_set_on_failure() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password",
    });

    // NOTE: Act 1
    let response = app.post_login(&login_body).await;
    let flash_message = response
        .cookies()
        .find(|c| c.name() == "_flash")
        .expect("Failed to retreive cookie by provided name");

    // NOTE: Assert 1
    assert_on_redirect(&response, "/login");
    assert_eq!(flash_message.value(), "Authentication Failed.");

    // NOTE: Act 2
    let login_html_text = app.get_login_html().await;
    assert!(login_html_text.contains(r#"Authentication Failed."#));
}
