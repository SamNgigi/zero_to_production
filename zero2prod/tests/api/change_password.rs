use uuid::Uuid;

use crate::helpers::{assert_on_redirect, spawn_app};

#[tokio::test]
async fn error_flash_message_is_set_on_incorrect_current_password() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act & Assert 1 - Successful Login.
    let login_request = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password
    });
    let response = app.post_login(&login_request).await;
    assert_on_redirect(&response, "/admin_dashboard");

    // NOTE: Act & Assert 2 - Redirect to admin/change_password
    let new_password = Uuid::new_v4().to_string();
    let change_password_request = serde_json::json!({
        "current_password": Uuid::new_v4().to_string(),
        "new_password": &new_password,
        "confirm_password": &new_password
    });
    let response = app.post_change_password(&change_password_request).await;
    assert_on_redirect(&response, "/admin/change_password");

    // NOTE: Act & Assert 3 - Flash Error Message Rendered.
    let change_password_html = app.get_change_password_html().await;
    assert!(change_password_html.contains("<p><i>The current password is incorrect.</i></p>"));
}

#[tokio::test]
async fn error_flash_message_is_set_on_new_password_fields_mismatch() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act & Assert 1 - Successful Login.
    let response = app
        .post_login(&serde_json::json!({
            "username": app.test_user.username,
            "password": app.test_user.password
        }))
        .await;

    assert_on_redirect(&response, "/admin_dashboard");

    // NOTE: Act & Assert 2 - Redirect Due to Password Mismatch
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": app.test_user.password,
            "new_password": Uuid::new_v4().to_string(),
            "confirm_password": Uuid::new_v4().to_string()
        }))
        .await;
    dbg!(&response);
    assert_on_redirect(&response, "/admin/change_password");

    // NOTE: Act & Assert 3 - Error Flash Message Rendered
    let change_password_html = app.get_change_password_html().await;
    assert!(change_password_html.contains(
        "<p><i>New Password and Confirm Password fields do not match. Fields must match.</i></p>"
    ));
}

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
