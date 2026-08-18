use uuid::Uuid;

use crate::common::{assert_on_redirect, spawn_app};

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
