use secrecy::SecretString;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::LazyLock;
use tokio::net::TcpListener;
use uuid::Uuid;

use zero2prod::{
    config::{DBSettings, get_config},
    email_client::EmailClient,
    startup as z2p,
};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }
}

static TRACING: LazyLock<()> = LazyLock::new(|| {});

pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    // setup `listener`
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");

    // Extract port and build app address
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // get config
    let mut config = get_config().expect("Failed to read configuration");

    // setup `connection_pool`
    config.db.db_name = format!("newsletter_axum_test_db_{}", Uuid::now_v7());
    let connection_pool = configure_db(&config.db).await;
    let connection_pool_clone = connection_pool.clone();

    // setup `email_client`
    let base_url = config.email_client.base_url();
    let sender_email = config
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let timeout = config.email_client.timeout();
    let email_client = EmailClient::new(
        base_url,
        sender_email,
        config.email_client.authorization_token,
        timeout,
    );

    // run app in a tokio asynchronous task
    tokio::spawn(async move { z2p::run(listener, connection_pool_clone, email_client).await });

    // return TestApp
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

async fn configure_db(config: &DBSettings) -> PgPool {
    // db setup
    let maintainence_settings = DBSettings {
        db_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: SecretString::new("password".into()),
        ..config.clone()
    };

    let mut connection = PgConnection::connect_with(&maintainence_settings.connect_options())
        .await
        .expect("Failed to connect to postgres in test");

    // create db
    connection
        .execute(format!(r#"CREATE DATABASE "{}""#, &config.db_name).as_str())
        .await
        .expect("Failed to create database in test");

    // connect to database
    let connection_pool = PgPool::connect_with(config.connect_options())
        .await
        .expect("Failed to connect with database in test");

    // migrate
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database in test");

    connection_pool
}
