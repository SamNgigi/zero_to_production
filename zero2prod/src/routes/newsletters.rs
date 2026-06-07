use actix_web::HttpResponse;

#[tracing::instrument(name = "Publish newsletter")]
pub async fn publish_newsletter() -> HttpResponse {
    HttpResponse::Ok().finish()
}
