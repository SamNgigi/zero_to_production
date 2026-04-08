use sqlx::{Connection, Executor, PgConnection, PgPool};
use tokio::net::TcpListener as TokioTcpListener;
use uuid::Uuid;
use zero2prod::{
    config::{DBSettings, get_config},
    startup as z2p, telemetry,
};

#[tokio::test]
async fn test_subscribe_returns_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    // Act
    let body = "username=lei%20yin&email=lei_yin_loo%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");
    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, username FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved request.");

    assert_eq!(saved.email, "lei_yin_loo@gmail.com");
    assert_eq!(saved.username, "lei yin");
}

#[tokio::test]
async fn test_subscribe_returns_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("username=lei%20yin", "missing the email"),
        ("email=lei_yin_loo%40gmail.com", "missing the name"),
        ("", "missing both username and email"),
    ];
    for (invalid_body, err_msg) in test_cases {
        // Act
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");
        // Assert
        assert_eq!(
            // Axum returns HTTP-semantically correct choice
            // - 422 Unprocessable Entity (I understood the content type but body is invalid)
            422,
            response.status().as_u16(),
            // Additional customized error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            err_msg
        )
    }
}

#[tokio::test]
async fn test_health_check() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute requests.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

use once_cell::sync::Lazy;

// INFO: Ensuring that the `tracing` stack is only initialized once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    /*
     * INFO:
     * We cannot assign the output of `get_subscriber` to a variable based on the
     * value `TEST_LOG` because the sink is part of the type returned by
     * `get_subscriber`, therefore they are not the same type. We could work around
     * it, but below is the most straight-forward way of moving forward
     */
    let default_filter_level = "info".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = telemetry::get_tracing_subscriber(default_filter_level, std::io::stdout);
        telemetry::init_tracing_subscriber(subscriber);
    } else {
        let subscriber = telemetry::get_tracing_subscriber(default_filter_level, std::io::sink);
        telemetry::init_tracing_subscriber(subscriber);
    }
});

#[derive(Debug)]
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    // INFO: Telemetry Test setup
    Lazy::force(&TRACING);

    // INFO: App Confguration
    let listener = TokioTcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");

    // Retrieving the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_config().expect("Failed to read configuration");
    configuration.db.db_name = format!("newsletter_test_db_{}", Uuid::now_v7());

    let connection_pool = configure_db(&configuration.db).await;
    let db_pool = connection_pool.clone();

    // INFO: Run app as async block/future in test context
    tokio::spawn(async move {
        z2p::run(listener, db_pool)
            .await
            .expect("Failed to run app in test");
    });

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_db(config: &DBSettings) -> PgPool {
    // Create the DB
    let mut connection = PgConnection::connect(&config.connection_string_without_db_name())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.db_name).as_str())
        .await
        .expect("Failed to create database");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
