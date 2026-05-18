use crate::common::spawn_app;

#[tokio::test]
async fn confirmation_link_without_subscription_token_rejected_with_400(){
    let app = spawn_app().await;

    let response = reqwest::get(format!("{}/subscriptions/confirm", app.address))
        .await
        .expect("Failed to execute get subscriptions confirm request");

    assert_eq!(response.status().as_u16(), 400);
}
