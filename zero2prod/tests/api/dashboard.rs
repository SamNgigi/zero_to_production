use crate::common::{assert_on_redirect, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_access_admin_dashboard() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act
    let response = app.get_admin_dashboard().await;

    // NOTE: Assert
    assert_on_redirect(&response, "/login");
}
