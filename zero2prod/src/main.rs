use tokio::net::TcpListener as TokioTcpListener;

use zero2prod::startup as z2p;

#[tokio::main]
async fn main() {
    let listener = TokioTcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to port");
    z2p::run(listener).await.expect("Failed to run application");
}
