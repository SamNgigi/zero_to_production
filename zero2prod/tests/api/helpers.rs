use std::sync::LazyLock;

use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use secrecy::SecretString;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;

use zero2prod::{
    config::{DBSettings, get_config},
    startup::{Application, get_connection_pool},
    telemetry,
};

pub fn assert_on_redirect(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(
        response
            .headers()
            .get("Location")
            .expect("Failed to get location header."),
        location
    );
}

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
    pub email_server: MockServer,
    pub port: u16,
    pub test_user: TestUser,
    pub client: reqwest::Client, // New field
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub text: reqwest::Url,
}

impl TestApp {
    pub async fn get_change_password_form(&self) -> reqwest::Response {
        self.client
            .get(format!("{}/admin/change_password", self.address))
            .send()
            .await
            .expect("Failed to execute GET /admin/change_password request in test.")
    }
    pub async fn get_admin_dashboard_html(&self) -> String {
        self.get_admin_dashboard()
            .await
            .text()
            .await
            .expect("Failed to decode html to valid text.")
    }

    pub async fn get_admin_dashboard(&self) -> reqwest::Response {
        self.client
            .get(format!("{}/admin_dashboard", self.address))
            .send()
            .await
            .expect("Failed to execute GET /admin_dashboard request in test")
    }

    pub async fn get_login_html(&self) -> String {
        self.client
            .get(format!("{}/login", self.address))
            .send()
            .await
            .expect("Failed to execute GET /login request in test.")
            .text()
            .await
            .expect("Failed to decode html to valid text.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.client
            .post(format!("{}/login", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to execute login POST request in test.")
    }
    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        self.client
            .post(format!("{}/newsletters", &self.address))
            .basic_auth(&self.test_user.username, Some(&self.test_user.password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute newsletter post request in test")
    }

    pub fn get_confirmation_links(&self, request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            // Ensuring we aren't calling random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(body["HtmlBody"].as_str().unwrap());
        let text = get_link(body["TextBody"].as_str().unwrap());
        ConfirmationLinks { html, text }
    }

    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        self.client
            .post(format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp {
    // INFO: The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution
    LazyLock::force(&TRACING);

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .expect("Failed to build reqwest::Client in test.");

    let email_server = MockServer::start().await;

    let configuration = {
        let mut cfg = get_config().expect("Failed to read configuration");
        cfg.db.db_name = format!("newsletter_actix_test_db_{}", Uuid::now_v7());
        cfg.app.port = 0;
        cfg.email_client.base_url = email_server.uri();
        cfg
    };

    configure_db(&configuration.db).await;
    let app = Application::build(configuration.clone())
        .await
        .expect("Failed to build Application");
    let port = app.port();
    let address = format!("http://127.0.0.1:{}", port);
    let _task = tokio::spawn(app.run_until_stopped());
    // Return TestApp
    let test_app = TestApp {
        address,
        db_pool: get_connection_pool(&configuration.db),
        email_server,
        port,
        test_user: TestUser::generate(),
        client,
    };
    test_app.test_user.store(&test_app.db_pool).await;
    test_app
}

async fn configure_db(config: &DBSettings) -> PgPool {
    // Instantiating test db settings
    let maintainence_settings = DBSettings {
        db_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: SecretString::new("password".into()),
        ..config.clone()
    };

    // Making the connection
    let mut connection = PgConnection::connect_with(&maintainence_settings.connect_options())
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

pub struct TestUser {
    user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    fn generate() -> Self {
        let test_user = Uuid::now_v7();
        Self {
            user_id: test_user,
            username: test_user.to_string(),
            // password: test_user.to_string(),
            password: "everythinghastostartfromsomewhere".into(),
        }
    }

    async fn store(&self, db_pool: &PgPool) {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(self.password.as_bytes(), &salt)
            .expect("Failed to get password hash")
            .to_string();

        sqlx::query!(
            r#"
                INSERT INTO users (user_id, username, password_hash)
                VALUES ($1, $2, $3);
            "#,
            self.user_id,
            self.username,
            password_hash,
        )
        .execute(db_pool)
        .await
        .expect("Failed to create test user");
    }
}
