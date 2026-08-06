use crate::helpers::{assert_on_redirect, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_access_change_password_form() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act
    let response = app.get_change_password().await;

    // NOTE: Assert
    assert_on_redirect(&response, "/login");
}
