use axum_messages::MessagesManagerLayer;
use secrecy::{ExposeSecret, SecretString};
use std::{sync::Arc, time::Duration};
use tower_sessions::{
    Expiry, SessionManagerLayer,
    cookie::{Key, SameSite},
};
use tower_sessions_redis_store::{
    RedisStore,
    fred::interfaces::ClientLike,
    fred::prelude::{Config, Pool},
};

use crate::{
    config::{DBSettings, Settings},
    email_client::EmailClient,
    routes::{
        ErrorReport, confirm, greet, health_check, login, login_form, publish_newsletter, subscribe,
    },
};

use axum::{
    Router, http,
    routing::{get, post},
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::{net::TcpListener as TokioTcpListener, signal};
use tower_http::trace::TraceLayer;
use tracing::{Level, Span, field::Empty};
use uuid::Uuid;

pub struct Application {
    port: u16,
    listener: TokioTcpListener,
    router: Router,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
        // Setup `listener` and extract `port`
        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = TokioTcpListener::bind(address)
            .await
            .expect("Failed to bind to port in build");
        let port = listener.local_addr()?.port();

        // Setup `connection_pool`
        let connection_pool = get_connection_pool(&config.db);

        let timeout = config.email_client.timeout();
        let email_client = EmailClient::new(
            config.email_client.base_url,
            config.email_client.sender_email,
            config.email_client.authorization_token,
            timeout,
        );

        // Setup `redis_pool`
        let redis_config = Config::from_url(config.redis_uri.expose_secret())
            .expect("Failed to configure redis uri.");
        let redis_pool =
            Pool::new(redis_config, None, None, None, 6).expect("Failed to create redis pool.");
        redis_pool.init().await.map_err(std::io::Error::other)?;

        let router = build_router(
            connection_pool,
            email_client,
            config.application.base_url,
            config.application.secret_key,
            redis_pool,
            config.application.secure_cookies,
        );

        Ok(Self {
            port,
            listener,
            router,
        })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        println!("👂 Listening on {}", &self.listener.local_addr()?);
        axum::serve(self.listener, self.router)
            .with_graceful_shutdown(shutdown_signal())
            .await
    }
}

pub fn get_connection_pool(db_config: &DBSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(db_config.connect_options())
}

pub struct ApplicationBaseUrl(pub String);

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub email_client: Arc<EmailClient>,
    pub base_url: Arc<ApplicationBaseUrl>,
}

fn build_router(
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    secret_key: SecretString,
    redis_pool: Pool,
    secure_cookies: bool,
) -> Router {
    let tracing_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &http::Request<_>| {
            let request_id = Uuid::now_v7();
            tracing::info_span!(
                "http_request",
                method = %request.method(),
                uri = %request.uri(),
                request_id = %request_id,
                status = Empty, // filled in on_response
                "error.message" = Empty, // filled in on_response
                "error.chain" = Empty, // filled in on_response
            )
        })
        .on_response(
            |response: &http::Response<_>, latency: Duration, span: &Span| {
                let status = response.status();
                span.record("status", status.as_u16());
                if let Some(report) = response.extensions().get::<ErrorReport>() {
                    span.record("error.message", report.message.as_str());
                    span.record("error.chain", report.details.as_str());
                }
                let latency = format!("{latency:?}");
                match () {
                    _ if status.is_server_error() => {
                        tracing::event!(Level::ERROR, %latency, "Internal Server Error.")
                    }
                    _ if status.is_client_error() => {
                        tracing::event!(Level::WARN, %latency, %status, "Client Side Error.")
                    }
                    _ => {
                        tracing::event!(Level::INFO, %latency, %status, "Request Completed.")
                    }
                }
            },
        )
        .on_failure(());

    let state = AppState {
        db_pool,
        email_client: Arc::new(email_client),
        base_url: Arc::new(ApplicationBaseUrl(base_url)),
    };
    let secret_key = Key::from(secret_key.expose_secret().as_bytes());
    let session_layer = SessionManagerLayer::new(RedisStore::new(redis_pool))
        .with_secure(secure_cookies)
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::Duration::minutes(10)))
        .with_signed(secret_key);

    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/", get(greet))
        .route("/{name}", get(greet))
        .route("/subscriptions/confirm", get(confirm))
        .route("/login", get(login_form))
        .route("/login", post(login))
        .route("/newsletters", post(publish_newsletter))
        .layer(MessagesManagerLayer)
        .layer(session_layer)
        .layer(tracing_layer)
        .with_state(state)
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
