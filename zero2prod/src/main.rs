use actix_web::{App, HttpServer, middleware::Logger, web};
use dotenv::dotenv;

// INFO: Place holder route
async fn index_page() -> &'static str {
    "Hello CRUD API!"
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/", web::get().to(index_page))
    })
    .bind("127.0.0.1:8001")?
    .run()
    .await
}
