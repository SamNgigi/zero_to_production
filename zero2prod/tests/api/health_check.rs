use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    let _app = spawn_app().await;
    todo!();
}
