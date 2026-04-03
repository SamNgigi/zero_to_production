use std::net::TcpListener;

use sqlx::{Connection, PgConnection};

use zero2prod::config::get_config;
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
    let address = spawn_app().await;
    // Perfoming HTTP requests against our application using reqwest
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

#[tokio::test]
async fn test_subscribe_returns_200_for_valid_form_data() {
    // Arrange
    let app_address = spawn_app().await;
    let configuration = get_config().expect("Failed to read configuration");
    let db_connection_string = configuration.db.connection_string();
    let db_connection = PgConnection::connect(&db_connection_string)
        .await
        .expect("Failed to connect to Postgres.");
    let client = reqwest::Client::new();

    // Act
    let body = "username=lei%20yin&email=lei_yin_loo%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut db_connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "lei_yin_loo@gmail.com");
    assert_eq!(saved.username, "lei yin");
}

#[tokio::test]
async fn test_subscribe_returns_400_when_data_is_missing() {
    // Arrange
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("username=lei%20yin", "missing the email"),
        ("email=lei_yin_low%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(format!("{}/subscriptions", &app_address))
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

// Launching our application in the background ~somehow~
async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    // Retrieving hte port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::startup::run(listener).expect("Failed to bind address");
    let _task = tokio::spawn(server);

    // Returning the application address to the caller
    format!("http://127.0.0.1:{}", port)
}
