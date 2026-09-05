use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::LazyLock;
use uuid::Uuid;
use wiremock::MockServer;

use zero2prod::{
    config::{DBSettings, get_config},
    email_client::EmailClient,
    issue_delivery_worker::{ExecutionOutcome, try_processing_task},
    startup::{Application, get_connection_pool},
    telemetry,
};

pub fn assert_on_redirect(response: &reqwest::Response, destination: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(
        response
            .headers()
            .get("Location")
            .expect("Failed to get location header"),
        destination
    );
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
    pub client: reqwest::Client,
    pub test_user: TestUser,
    pub email_client: EmailClient,
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub text: reqwest::Url,
}

impl TestApp {
    pub async fn dispatch_all_pending_emails(&self) {
        loop {
            if let Ok(ExecutionOutcome::EmptyQueue) =
                try_processing_task(&self.db_pool, &self.email_client).await
            {
                break;
            }
        }
    }

    pub async fn post_publish_newsletter<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.client
            .post(format!("{}/admin/publish_newsletter", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to execute POST /admin/publish_newsletter request in test.")
    }

    pub async fn get_publish_newsletter_html(&self) -> String {
        self.get_publish_newsletter()
            .await
            .text()
            .await
            .expect("Failed to decode HTML to valid String text in test.")
    }

    pub async fn get_publish_newsletter(&self) -> reqwest::Response {
        self.client
            .get(format!("{}/admin/publish_newsletter", &self.address))
            .send()
            .await
            .expect("Failed to execute GET /admin/publish_newsletter request in test.")
    }

    pub async fn post_change_password<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.client
            .post(format!("{}/admin/change_password", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to execute POST /admin/change_password request in test.")
    }

    pub async fn get_change_password_html(&self) -> String {
        self.get_change_password()
            .await
            .text()
            .await
            .expect("Failed to decode HTML to String text in test.")
    }

    pub async fn get_change_password(&self) -> reqwest::Response {
        self.client
            .get(format!("{}/admin/change_password", &self.address))
            .send()
            .await
            .expect("Failed to execute GET /admin/change_password request in test.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.client
            .post(format!("{}/admin/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute POST /admin/logout/ request in test.")
    }

    pub async fn get_admin_dashboard_html(&self) -> String {
        self.get_admin_dashboard()
            .await
            .text()
            .await
            .expect("Failed to decode HTML to valid text in test.")
    }

    pub async fn get_admin_dashboard(&self) -> reqwest::Response {
        self.client
            .get(format!("{}/admin/dashboard", &self.address))
            .send()
            .await
            .expect("Failed to execute GET /admin/dashboard request in test.")
    }

    pub async fn get_login_html(&self) -> String {
        self.get_login()
            .await
            .text()
            .await
            .expect("Failed to decode HTML to valid String text in test.")
    }

    pub async fn get_login(&self) -> reqwest::Response {
        self.client
            .get(format!("{}/login", &self.address))
            .send()
            .await
            .expect("Failed to execute GET /login request in test")
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
            .expect("Failed to execute POST /login request in test.")
    }

    pub fn get_confirmation_links(&self, req: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&req.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
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
            .expect("Failed to execute request")
    }
}

pub struct TestUser {
    user_id: Uuid,
    pub username: String,
    pub password: SecretString,
}

impl TestUser {
    pub async fn login(&self, app: &TestApp) {
        app.post_login(&serde_json::json!({
            "username": self.username,
            "password": self.password.expose_secret(),
        }))
        .await;
    }

    fn generate() -> Self {
        let test_user = Uuid::now_v7();
        Self {
            user_id: test_user,
            username: test_user.to_string(),
            password: "everythinghastostartfromsomewhere".into(),
        }
    }

    async fn store(&self, db_pool: &PgPool) {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(self.password.expose_secret().as_bytes(), &salt)
            .expect("Failed to generate password hash in test.")
            .to_string();

        sqlx::query!(
            r#"
                INSERT INTO users (user_id, username, password_hash)
                VALUES ($1, $2, $3)
            "#,
            self.user_id,
            self.username,
            password_hash,
        )
        .execute(db_pool)
        .await
        .expect("Failed to execute SQL query to store test user credentials.");
    }
}

// INFO: Ensuring that the `tracing` stack is only initialized once using `std::sync::LazyLock`.
// Replaces `once_cell::sync::Lazy`.
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = telemetry::get_tracing_subscriber(default_filter_level, std::io::stdout);
        telemetry::init_tracing_subscriber(subscriber);
    } else {
        let subscriber = telemetry::get_tracing_subscriber(default_filter_level, std::io::sink);
        telemetry::init_tracing_subscriber(subscriber);
    }
});

pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .expect("Failed to build reqwest::Client in test.");

    let email_server = MockServer::start().await;

    // get config
    let configuration = {
        let mock_base_url = reqwest::Url::parse(&email_server.uri()).unwrap();
        let mut config = get_config().expect("Failed to read configuration");
        config.db.db_name = format!("newsletter_axum_test_db_{}", Uuid::now_v7());
        config.application.port = 0;
        config.email_client.base_url = mock_base_url;
        config
    };

    configure_db(&configuration.db).await;

    let app = Application::build(configuration.clone())
        .await
        .expect("Failed to build application in test");

    let port = app.port();
    let address = format!("http://127.0.0.1:{}", port);
    // run app in a tokio asynchronous task
    tokio::spawn(app.run_until_stopped());

    // return TestApp
    let test_app = TestApp {
        address,
        db_pool: get_connection_pool(&configuration.db),
        email_server,
        port,
        client,
        test_user: TestUser::generate(),
        email_client: configuration.email_client.client(),
    };

    test_app.test_user.store(&test_app.db_pool).await;
    test_app
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
