use axum::{Router, extract::Path, http::StatusCode, response::IntoResponse, routing::get};
use tokio::{net::TcpListener as TokioTcpListener, signal};

async fn greet(name: Option<Path<String>>) -> String {
    let name = name.map(|Path(n)| n).unwrap_or_else(|| "World".to_string());
    format!("Hello {name}!")
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn run(listener: TokioTcpListener) -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/", get(greet))
        .route("/{name}", get(greet));

    println!("👂 Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL + c handler");
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
