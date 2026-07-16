use crate::helpers::spawn_app;

fn assert_on_redirect(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(
        response
            .headers()
            .get("Location")
            .expect("Location was set"),
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

    // NOTE: Act
    let response = app.post_login(&login_body).await;
    let flash_msg = response
        .cookies()
        .find(|c| c.name() == "_flash")
        .expect("Failed to find cookie by provided name");

    //NOTE: Assert
    assert_on_redirect(&response, "/login");
    assert_eq!(flash_msg.value(), "Authentication Failed.");
}
