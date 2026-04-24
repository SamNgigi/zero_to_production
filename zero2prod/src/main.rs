use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

use zero2prod::{
    config::get_config,
    email_client::EmailClient,
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
    // No longer async, given that we don't actually try to connect. We use connect_lazy instead
    let connection_pool = PgPoolOptions::new().connect_lazy_with(config.db.connect_options());
    // Building an `EmailClient` using `config`
    let sender = config
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let timeout = config.email_client.timeout();
    let email_client = EmailClient::new(
        config.email_client.base_url,
        sender,
        config.email_client.authorization_token,
        timeout,
    );

    let address = format!("{}:{}", config.app.host, config.app.port);
    let listener = TcpListener::bind(address)
        .unwrap_or_else(|_| panic!("Failed to bind to port: {}", config.app.port));

    // INFO: Run App
    run(listener, connection_pool, email_client)?.await?;

    Ok(())
}
