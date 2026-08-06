use uuid::Uuid;

use crate::helpers::{assert_on_redirect, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_post_to_change_password() {
    // NOTE: Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4();
    let change_password_request = serde_json::json!({
        "current_password": app.test_user.password,
        "new_password": &new_password,
        "confirm_password": &new_password
    });

    // NOTE: Act
    let response = app.post_change_password(&change_password_request).await;

    // NOTE: Assert
    assert_on_redirect(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_access_change_password_form() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act
    let response = app.get_change_password().await;

    // NOTE: Assert
    assert_on_redirect(&response, "/login");
}
