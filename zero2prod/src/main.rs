use tokio::net::TcpListener as TokioTcpListener;
use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt};

use zero2prod::{
    config::{create_pool, get_config},
    startup as z2p,
};

#[tokio::main]
async fn main() {
    // INFO: Falling back to info-level or above for all spans
    LogTracer::init().expect("Failed to set logger");
    // if the RUST_LOG environment variable has not been set.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace"));
    let formatting_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_target(true)
        .with_thread_ids(true);

    // INFO: The `with` method is provided by `SubscriberExt`, an extension
    // trait for `Subscriber` exposed by `tracing_subscriber`
    let subscriber = Registry::default().with(env_filter).with(formatting_layer);
    // INFO: `set_global_subscriber` can be used by applications to specify
    // what subscriber should be used to process spans.
    set_global_default(subscriber).expect("Failed to set subscriber");
    // We panic if we can't read configuration
    let config = get_config().expect("Failed to read configuration");
    let connection_pool = create_pool(&config.db)
        .await
        .expect("Failed to connect to Postgres");
    let address = format!("127.0.0.1:{}", config.app_port);
    let listener = TokioTcpListener::bind(address)
        .await
        .expect("Failed to bind to port");
    z2p::run(listener, connection_pool)
        .await
        .expect("Failed to run application");
}
