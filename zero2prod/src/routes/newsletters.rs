use actix_web::{HttpResponse, web};

#[derive(Debug, serde::Deserialize)]
pub struct BodyData {
    _title: String,
    _content: Content,
}

#[derive(Debug, serde::Deserialize)]
pub struct Content {
    _html: String,
    _plain: String,
}

#[tracing::instrument(name = "Publish newsletter")]
pub async fn publish_newsletter(_body: web::Json<BodyData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
