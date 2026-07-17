use crate::helpers::spawn_app;

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
    let flash_message = response
        .cookies()
        .find(|c| c.name() == "_flash")
        .expect("Failed to retreive cookie by provided name");

    // NOTE: Assert
    assert_eq!(flash_message.value(), "Authentication Failed.");
}
