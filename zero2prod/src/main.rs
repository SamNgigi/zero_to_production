use axum::{Router, extract::Path, routing::get};
use tokio::signal;

async fn greet(name: Option<Path<String>>) -> String {
    let name = name.map(|Path(n)| n).unwrap_or_else(|| "World".to_string());
    format!("Hello {name}!")
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(greet))
        .route("/{name}", get(greet));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("👂 Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install CTRL+c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        /* signal::unix::signal(signal::unix::SignalKind::terminate())
        .expect("failed to install signal handler")
        .recv()
        .await; */
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // Wating for the first signal to arrive
    tokio::select! {
        _ = ctrl_c => println!("🛑 Received CTRL + C, shutting down.."),
        _ = terminate => println!("🛑 Received SIGTERM, shutting down.."),
    }
}
