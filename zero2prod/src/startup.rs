use std::time::Duration;

use crate::routes::{greet, health_check, subscribe};

use axum::{
    Router, http,
    routing::{get, post},
};
use sqlx::PgPool;
use tokio::{net::TcpListener as TokioTcpListener, signal};
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::Span;
use uuid::Uuid;

pub async fn run(
    listener: TokioTcpListener,
    db_pool: PgPool, // New param
) -> Result<(), std::io::Error> {
    let tracing_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &http::Request<_>| {
            let request_id = Uuid::now_v7();
            tracing::info_span!(
                "http_request",
                method = %request.method(),
                uri = %request.uri(),
                request_id = %request_id
            )
        })
        .on_response(
            |response: &http::Response<_>, latency: Duration, _span: &Span| {
                tracing::info!("response: {} {:?}", response.status(), latency)
            },
        )
        .on_failure(
            |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                tracing::error!("error: {}", error)
            },
        );

    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/", get(greet))
        .route("/{name}", get(greet))
        .layer(tracing_layer)
        .with_state(db_pool);

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
    let terminate = std::future::pending::<()>();

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // Wating for the first signal to arrive
    tokio::select! {
        _ = ctrl_c => println!("🛑 Received CTRL + C, shutting down.."),
        _ = terminate => println!("🛑 Received SIGTERM, shutting down.."),
    }
}
