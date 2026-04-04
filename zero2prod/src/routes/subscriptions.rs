use axum::{Form, http::StatusCode, response::IntoResponse};

#[derive(serde::Deserialize)]
pub struct FormData {
    _email: String,
    _username: String,
}

pub async fn subscribe(Form(_form): Form<FormData>) -> impl IntoResponse {
    StatusCode::OK
}
