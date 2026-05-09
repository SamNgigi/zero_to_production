use crate::config::{DBSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{confirm, greet, health_check, subscribe};
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
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
        let server = run(listener, db_pool, email_client)?;

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

fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient, // New param
) -> Result<Server, std::io::Error> {
    // INFO: `web::Data::new` allows us to wrap the arguments provided
    // as a `Arc` type that can be shared application wide between threads
    // as opposed to a full copy.
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            // Registering connection as part of applicaton state
            .app_data(db_pool.clone())
            // Regeistering email client as part of application state
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
