use tokio::net::TcpListener as TokioTcpListener;

use zero2prod::{
    config::{create_pool, get_config},
    startup as z2p, telemetry,
};

#[tokio::main]
async fn main() {
    // INFO: Telemetry setup
    let subscriber = telemetry::get_tracing_subscriber("info".into());
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
