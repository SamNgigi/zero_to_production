use tokio::net::TcpListener as TokioTcpListener;

use zero2prod::{
    config::{create_pool, get_config},
    startup as z2p,
};

#[tokio::main]
async fn main() {
    // We panic if we can't read configuration.
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
