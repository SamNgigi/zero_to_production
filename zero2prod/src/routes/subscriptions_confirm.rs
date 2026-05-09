use actix_web::{HttpResponse, web};

#[derive(serde::Deserialize)]
pub struct Params {
    _subscription_token: String,
}

#[tracing::instrument(name = "Confirm pending subscriber", skip(_params))]
pub async fn confirm(_params: web::Query<Params>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
