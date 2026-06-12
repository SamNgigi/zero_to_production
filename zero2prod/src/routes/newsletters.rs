use axum::{Json, http::StatusCode, response::IntoResponse};

#[derive(serde::Deserialize)]
pub struct BodyData {
    _title: String,
    _content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    _html: String,
    _plain: String,
}

#[tracing::instrument(name = "Publish newsletter issue", skip(_body))]
pub async fn publish_newsletter(_body: Json<BodyData>) -> impl IntoResponse {
    StatusCode::OK
}
