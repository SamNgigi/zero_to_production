## Repetitive curl test
```curl
curl -d "username=lei%20yin&email=lei_yin_loo%40gmail.com" http://127.0.0.1:8000/subscriptions

```
```Rust
// ---------------------------------------------------
// src/main.rs
// ---------------------------------------------------

use tokio::net::TcpListener as TokioTcpListener;

use zero2prod::{
    config::{create_pool, get_config},
    startup as z2p, telemetry,
};

#[tokio::main]
async fn main() {
    // INFO: Telemetry setup
    let subscriber = telemetry::get_tracing_subscriber(
        "info".into(),
        std::io::stdout, // sink when app is running
    );
    telemetry::init_tracing_subscriber(subscriber);

    // INFO: App configuration
    let config = get_config().expect("Failed to read configuration");
    let connection_pool = create_pool(&config.db)
        .await
        .expect("Failed to connect to Postgres");
    let address = format!("127.0.0.1:{}", config.app_port);
    let listener = TokioTcpListener::bind(address)
        .await
        .expect("Failed to bind to port");

    // INFO: Running App
    z2p::run(listener, connection_pool)
        .await
        .expect("Failed to run application");
}

// ---------------------------------------------------
// src/startup.rs
// ---------------------------------------------------
use crate::routes::{greet, health_check, subscribe};

use axum::{
    Router, http,
    routing::{get, post},
};
use sqlx::PgPool;
use tokio::{net::TcpListener as TokioTcpListener, signal};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

pub async fn run(
    listener: TokioTcpListener,
    db_pool: PgPool, // New param
) -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/", get(greet))
        .route("/{name}", get(greet))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &http::Request<_>| {
                let request_id = Uuid::now_v7();
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                    request_id = %request_id
                )
            }),
        )
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

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // Wating for the first signal to arrive
    tokio::select! {
        _ = ctrl_c => println!("🛑 Received CTRL + C, shutting down.."),
        _ = terminate => println!("🛑 Received SIGTERM, shutting down.."),
    }
}


// ---------------------------------------------------
// src/routes/subscriptions.rs
// ---------------------------------------------------
use axum::{Form, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    username: String,
}

/* INFO:
 * `subscribe` orchestrates the work to be done by calling the required
 * routines and translates their outcomes into the proper response
 * according to the rules and conventions of the HTTP protocol
 * */
#[tracing::instrument(
    name = "Adding a new subscriber"
    skip(db_pool, form),
    fields(
        subscriber_email = %form.email,
        subscriber_username = %form.username
    )
)]
pub async fn subscribe(
    State(db_pool): State<PgPool>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    match insert_subscriber(&db_pool, &form).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/* INFO:
 * `insert_subscriber` takes care of the database logic and it has no
 * awareness of the surrounding web framework. Easily portable
 * */

#[tracing::instrument(
    name = "Saving new subscriber details in the database"
    skip(pool, form)
)]
async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, username, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::now_v7(),
        form.email,
        form.username,
        Utc::now(),
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
        /* INFO:
         * Using the `?` operator to return early
         * if the function failed, returning a sqlx::Error
         * We will talk about error handling in depth later!
         * */
    })?;

    Ok(())
}
```
