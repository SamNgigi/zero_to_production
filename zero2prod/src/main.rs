use tokio::net::TcpListener as TokioTcpListener;

use zero2prod::{
    config::{create_pool, get_config},
    email_client::EmailClient,
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
    tracing::info!("Connecting to database at host: {}", config.db.host);
    let connection_pool = create_pool(&config.db);
    let sender_email = config
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let timeout = config.email_client.timeout();
    let email_client = EmailClient::new(
        config.email_client.base_url,
        sender_email,
        config.email_client.authorization_token,
        timeout,
    );

    let address = format!("{}:{}", config.app.host, config.app.port);
    let listener = TokioTcpListener::bind(address)
        .await
        .expect("Failed to bind to port");

    // INFO: Running App
    z2p::run(listener, connection_pool, email_client)
        .await
        .expect("Failed to run application");
}
