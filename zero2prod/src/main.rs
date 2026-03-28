use zero2prod::run;

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to port");
    run(listener).await.expect("Failed to run application");
}
