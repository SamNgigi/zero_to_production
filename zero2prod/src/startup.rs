use crate::routes::{greet, health_check, subscribe};
use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};
use env_logger::Env;
use sqlx::PgPool;
use std::net::TcpListener;

pub fn run(
    listener: TcpListener,
    db_pool: PgPool, // New param
) -> Result<Server, std::io::Error> {
    /*
     * INFO:
     * `init` does call `set_logger`, so this is all we need to do.
     * We are falling back to printing all logs at info-level or above
     * if the RUST_LOG environment variable has not been set.
     */
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

    let db_pool = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
            .route("/subscriptions", web::post().to(subscribe))
            // Registering connection as part of applicaton state
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
