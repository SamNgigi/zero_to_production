use crate::common::spawn_app;

#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_invalid() {
    let _test_cases = [
        ("username=&email=lei_yin_loo%40gmail.com", "empty name"),
        ("username=lei&email=", "empty email"),
        (
            "username=lei&email=definitely-not-an-email",
            "invalid email",
        ),
    ];
    todo!()
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let _test_cases = [
        ("username=lei%20yin", "missing the email"),
        ("email=lei_yin_loo%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    todo!()
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    // Act
    // Assert
}
