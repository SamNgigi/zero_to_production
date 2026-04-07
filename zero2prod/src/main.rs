use sqlx::PgPool;
use std::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt};

use zero2prod::{config::get_config, startup::run};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    // Redirecting all `log`'s events to our subscriber
    LogTracer::init().expect("Failed to set logger");
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = fmt::layer()
        .json() // Structured JSON output
        .with_current_span(true)
        .with_span_list(true)
        .with_target(true)
        .with_thread_ids(true);
    let subscriber = Registry::default().with(env_filter).with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set subscriber");

    // We panic if we can't read configuration.
    let config = get_config().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&config.db.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", config.app_port);
    let listener = TcpListener::bind(address)
        .unwrap_or_else(|_| panic!("Failed to bind to port: {}", config.app_port));
    run(listener, connection_pool)?.await
}
