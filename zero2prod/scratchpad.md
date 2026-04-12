## Repetitive curl test
```curl
curl -d "username=lei%20yin&email=lei_yin_loo%40gmail.com" http://127.0.0.1:8000/subscriptions
curl -v https://zero2prod-axum-impl.fly.dev/health_check -> SUCCESSFUL
curl -d "username=lei%20yin&email=lei_yin_loo%40gmail.com" https://zero2prod-axum-impl.fly.dev/subscriptions -> SUCCESSFUL

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
    let connection_pool = create_pool(&config.db);
    connection_pool
        .acquire()
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", config.app.port);
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

// ---------------------------------------------------
// src/telemetry.rs
// ---------------------------------------------------
use tracing::subscriber::{Subscriber, set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::{
    EnvFilter, Registry,
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
};

/// Compose multiple layers into a `tracing`'s subscriber
///
/// # Implementation Notes
///
/// We're using `impl Subscriber` as the return type to avoid having to
/// spell out the actual type of the returned subscriber, which is
/// indeed quite complex
/// We nee to explicitly call out that the returned subscriver is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// later on.
pub fn get_tracing_subscriber<Sink>(env_filter: String, sink: Sink) -> impl Subscriber + Sync + Send
where
    // INFO: This "weird" syntax is a higher-ranked trait bound (HRTB)
    // It basically means that Sink implements the `MakeWriter` trait
    // for all choices of the lifetime parameter `'a`
    // Check out https://doc.rust-lang.org/nomicon/hrtb.html
    // for more details
    Sink: for<'a> MakeWriter<'a> + Sync + Send + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = fmt::layer()
        .json()
        .with_writer(sink)
        .with_current_span(true)
        .with_target(true);
    Registry::default().with(env_filter).with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_tracing_subscriber(subscriber: impl Subscriber + Sync + Send) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set tracing subscriber");
}


// ---------------------------------------------------
// src/config.rs
// ---------------------------------------------------
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use secrecy::{ExposeSecret, SecretString};

pub enum Environment {
    DEVELOPMENT,
    PRODUCTION,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::DEVELOPMENT => "development",
            Environment::PRODUCTION => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        match val.to_lowercase().as_str() {
            "development" => Ok(Self::DEVELOPMENT),
            "production" => Ok(Self::PRODUCTION),
            other => Err(format!(
                "{} is not a supported environment. \
                    Use either `development` or `production`.",
                other
            )),
        }
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");
    let environment_filename = format!("{}.yaml", environment.as_str());

    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .build()?;
    // Try to convert the read config values into our Setting type
    settings.try_deserialize::<Settings>()
}

#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    pub db: DBSettings,
    pub app: AppSettings,
}

#[derive(Debug, serde::Deserialize)]
pub struct AppSettings {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct DBSettings {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub db_name: String,
    pub max_connections: u32,
}

impl DBSettings {
    pub fn connection_string(&self) -> SecretString {
        SecretString::new(
            format!(
                "postgres://{}:{}@{}:{}/{}",
                self.username,
                self.password.expose_secret(),
                self.host,
                self.port,
                self.db_name
            )
            .into(),
        )
    }

    pub fn connection_string_without_db_name(&self) -> SecretString {
        SecretString::new(
            format!(
                "postgres://{}:{}@{}:{}",
                self.username,
                self.password.expose_secret(),
                self.host,
                self.port
            )
            .into(),
        )
    }
}

pub fn create_pool(cfg: &DBSettings) -> PgPool {
    let options = PgConnectOptions::new()
        .username(&cfg.username)
        .password(cfg.password.expose_secret())
        .port(cfg.port)
        .host(&cfg.host)
        .database(&cfg.db_name);

    PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .connect_lazy_with(options)
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
