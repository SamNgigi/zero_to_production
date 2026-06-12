use axum::{http::StatusCode, response::IntoResponse};

#[tracing::instrument(name = "Publish newsletter issue")]
pub async fn publish_newsletter() -> impl IntoResponse {
    StatusCode::OK
}
