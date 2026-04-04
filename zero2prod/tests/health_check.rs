use sqlx::PgPool;
use tokio::net::TcpListener as TokioTcpListener;
use zero2prod::{config::get_config, startup as z2p};

#[tokio::test]
async fn test_subscribe_returns_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    // Act
    let body = "_username=lei%20yin&_email=lei_yin_loo%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");
    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn test_subscribe_returns_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("_username=lei%20yin", "missing the email"),
        ("_email=lei_yin_loo%40gmail.com", "missing the name"),
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

#[derive(Debug)]
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TokioTcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");

    // Retrieving the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let configuration = get_config().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.db.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let db_pool = connection_pool.clone();
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
