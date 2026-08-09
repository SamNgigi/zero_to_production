use actix_session::{SessionMiddleware, storage::RedisSessionStore};
use actix_web::{App, HttpServer, cookie::Key, dev::Server, web};
use actix_web_flash_messages::{FlashMessagesFramework, storage::CookieMessageStore};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::{
    authentication::reject_anonymous_users,
    config::{DBSettings, Settings},
    email_client::EmailClient,
    routes::{
        admin_dashboard, change_password, change_password_form, confirm, greet, health_check, home,
        login, login_form, logout, publish_newsletter, subscribe,
    },
};

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, anyhow::Error> {
        let db_pool = get_connection_pool(&config.db);

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

        let listener = TcpListener::bind(format!("{}:{}", config.app.host, config.app.port))
            .unwrap_or_else(|_| panic!("Failed to bind to port {}", config.app.port));
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            db_pool,
            email_client,
            config.app.base_url,
            config.app.secret_key,
            config.redis_uri,
        )
        .await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    // More expressive name that makes it clear that
    // this function only returns when application is stopped
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(db_config: &DBSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(db_config.connect_options())
}

pub struct ApplicationBaseUrl(pub String);

async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    secret_key: SecretString,
    redis_uri: SecretString, // New param
) -> Result<Server, anyhow::Error> {
    // INFO: `web::Data::new` allows us to wrap the arguments provided
    // as a `Arc` type that can be shared application wide between threads
    // as opposed to a full copy.
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let secret_key = Key::from(secret_key.expose_secret().as_bytes());
    let cookie_storage = CookieMessageStore::builder(secret_key.clone()).build();
    let flash_messages = FlashMessagesFramework::builder(cookie_storage).build();
    let session_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;
    let server = HttpServer::new(move || {
        App::new()
            .wrap(flash_messages.clone())
            .wrap(SessionMiddleware::new(
                session_store.clone(),
                secret_key.clone(),
            ))
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/", web::get().to(greet))
            .route("/home", web::get().to(home))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .route("/{name}", web::get().to(greet))
            .service(
                web::scope("/admin")
                    .wrap(actix_web::middleware::from_fn(reject_anonymous_users))
                    .route("/dashboard", web::get().to(admin_dashboard))
                    .route("/change_password", web::get().to(change_password_form))
                    .route("/change_password", web::post().to(change_password))
                    .route("/logout", web::post().to(logout)),
            )
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/newsletters", web::post().to(publish_newsletter))
            // Registering connection as part of applicaton state
            .app_data(db_pool.clone())
            // Registering email client as part of application state
            .app_data(email_client.clone())
            // Registering base_url as part of application state
            .app_data(base_url.clone())
            // Registering secret_key as part of application state
            .app_data(secret_key.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
