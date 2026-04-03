use actix_web::{HttpResponse, web};
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub username: String,
}

pub async fn subscribe(
    _form: web::Form<FormData>,
    _pool: web::Data<PgPool>, // Retrieving a connection from App State
) -> HttpResponse {
    HttpResponse::Ok().finish()
}
