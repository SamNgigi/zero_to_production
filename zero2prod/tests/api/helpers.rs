use std::net::TcpListener;
use std::sync::LazyLock;

use secrecy::SecretString;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use zero2prod::{
    config::{DBSettings, get_config},
    email_client::EmailClient,
    startup::run,
    telemetry,
};

// INFO: Ensuring that the `tracing` stack is only initialized once using `std::sync::LazyLock`.
// Replaces `once_cell::sync::Lazy`.
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();

    /* INFO:
     * We cannot assign the output of `get_subscriber` to a variable based on the value
     * `TEST_LOG` because the sink is part of the type returned by `get_subscriber`,
     * therefore they are not the same type. We could work around it, but this is the
     * most straigh-forward way of moving forward.
     * */
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = telemetry::get_subscriber(default_filter_level, std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber = telemetry::get_subscriber(default_filter_level, std::io::sink);
        telemetry::init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    // INFO: The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution
    LazyLock::force(&TRACING);

    // Building listener and capturing the listener port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut config = get_config().expect("Failed to read configuration");

    // Building connection_pool
    config.db.db_name = format!("newsletter_test_db_actix_{}", Uuid::now_v7());
    let connection_pool = configure_db(&config.db).await;

    // Building email client
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

    // Build server
    let server =
        run(listener, connection_pool.clone(), email_client).expect("Failed to bind address");
    let _task = tokio::spawn(server);

    // Return TestApp
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

async fn configure_db(config: &DBSettings) -> PgPool {
    // Instantiating test db settings
    let maintainenc_settings = DBSettings {
        db_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: SecretString::new("password".into()),
        ..config.clone()
    };

    // Making the connection
    let mut connection = PgConnection::connect_with(&maintainenc_settings.connect_options())
        .await
        .expect("Failed to connect to Postgres");

    // Creating the database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.db_name).as_str())
        .await
        .expect("Failed to create database");

    // Getting the db_pool connection
    let connection_pool = PgPool::connect_with(config.connect_options())
        .await
        .expect("Failed to connect to Postgres");

    // Making migrations
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
