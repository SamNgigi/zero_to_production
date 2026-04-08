use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

use zero2prod::{
    config::get_config,
    startup::run,
    telemetry::{self, init_subscriber},
};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    // INFO: Telemetry setup
    let subscriber = telemetry::get_subscriber("info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // INFO: App configuration
    let config = get_config().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(config.db.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", config.app_port);
    let listener = TcpListener::bind(address)
        .unwrap_or_else(|_| panic!("Failed to bind to port: {}", config.app_port));

    // INFO: Run App
    run(listener, connection_pool)?.await
}
