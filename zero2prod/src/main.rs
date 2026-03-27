use actix_web::{App, HttpServer, middleware::Logger, web};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};

// Model
#[derive(Serialize, Deserialize, Debug)]
struct BlogPost {
    id: i32,
    title: String,
    content: String,
    author: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct NewBlogPost {
    title: String,
    content: String,
    author: String,
}

// INFO: Place holder route
async fn index_page() -> &'static str {
    "Hello CRUD API!"
}

async fn create_blogpost() -> &'static str {
    "Hello CRUD API!"
}

async fn read_blogpost() -> &'static str {
    "Hello CRUD API!"
}

async fn update_blogpost() -> &'static str {
    "Hello CRUD API!"
}

async fn delete_blogpost() -> &'static str {
    "Hello CRUD API!"
}

async fn get_all_blogpost() -> &'static str {
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
