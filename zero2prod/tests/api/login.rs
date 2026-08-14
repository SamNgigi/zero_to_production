use uuid::Uuid;

use crate::common::{assert_on_redirect, spawn_app};

#[tokio::test]
async fn error_flash_message_is_set_on_login_failure() {
    // NOTE: Arrange
    let app = spawn_app().await;

    // NOTE: Act & Assert 1 - Redirect to Login on Unsuccessful authentication
    let response = app
        .post_login(&serde_json::json!({
            "username": Uuid::new_v4().to_string(),
            "password": Uuid::new_v4().to_string(),
        }))
        .await;
    dbg!(&response);
    assert_on_redirect(&response, "/login");

    // NOTE: Act & Assert 2 - Flash Error Message rendered
    let login_html = app.get_login_html().await;
    assert!(
        login_html
            .contains(r#"<p><i>Authentication failed. Invalid username or password.</i></p>"#)
    );

    // NOTE: Act & Assert 3 - Flash Message NOT rendered - Ephimeral flash message
    let login_html = app.get_login_html().await;
    assert!(
        !login_html
            .contains(r#"<p><i>Authentication failed. Invalid username or password.</i></p>"#)
    );
}
