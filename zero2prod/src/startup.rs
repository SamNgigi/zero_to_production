use crate::email_client::EmailClient;
use crate::routes::{greet, health_check, subscribe};
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(
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
            // Registering connection as part of applicaton state
            .app_data(db_pool.clone())
            // Regeistering email client as part of application state
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
