use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};

use uuid::Uuid;
use zero2prod::{
    config::{DBSettings, get_config},
    email_client::EmailClient,
    startup::run,
    telemetry::{self, init_subscriber},
};

/*
* `tokio::test` is the testing equivalent of `tokio::main`.
* It also spares us from having to specify the `#[test]` attribute.
*
* We can inspect what code gets generated using
* `cargo expand --test health_chect` (<- name of the test file)
* */
#[tokio::test]
async fn test_heath_check() {
    // Arrange
    let _s = Uuid::now_v7();
    let app = spawn_app().await;
    // Perfoming HTTP requests against our application using reqwest
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

/// Test to validate troublesome user input for name & email
#[tokio::test]
async fn subscribe_return_a_400_when_fields_are_present_but_invalid() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("username=&email=lei_yin_loo%40gmail.com", "empty name"),
        ("username=lei&email=", "empty email"),
        (
            "username=lei&email=definitely-not-an-email",
            "invalid email",
        ),
    ];

    for (body, description) in test_cases {
        // Act
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        )
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        // Act
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}

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
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, username FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

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
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customized error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}

use once_cell::sync::Lazy;

// INFO: Ensuring that the `tracing` stack is only initialized once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    // INFO: We cannot assign the output of `get_subscriber` to a variable based on the
    // value `TEST_LOG` because the sink is part of the type returned by
    // `get_subscriber`, therefore they are not the same type. We could work around
    // it, but this is the most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = telemetry::get_subscriber(default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = telemetry::get_subscriber(default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// Launching our application in the background ~somehow~
async fn spawn_app() -> TestApp {
    // INFO: The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    // Retrieving hte port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_config().expect("Failed to read configuration");
    // configuration.db.db_name = Uuid::now_v7().to_string();
    configuration.db.db_name = format!("newsletter_test_db_actix_{}", Uuid::now_v7());

    // Building email client for test
    let sender = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender,
        configuration.email_client.authorization_token,
        timeout,
    );

    let connection_pool = configure_db(&configuration.db).await;
    let server =
        run(listener, connection_pool.clone(), email_client).expect("Failed to bind address");
    let _task = tokio::spawn(server);

    // Returning the application address to the caller
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_db(config: &DBSettings) -> PgPool {
    let mut connection =
        PgConnection::connect(config.connection_string_without_db_name().expose_secret())
            .await
            .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect(config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
