use sqlx::PgPool;
use std::net::TcpListener;

use zero2prod::{config::get_config, startup::run};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
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
