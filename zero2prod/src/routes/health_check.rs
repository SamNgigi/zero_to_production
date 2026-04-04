use axum::{extract::Path, http::StatusCode, response::IntoResponse};

pub async fn greet(name: Option<Path<String>>) -> String {
    let name = name.map(|Path(n)| n).unwrap_or_else(|| "World".to_string());
    format!("Hello {name}!")
}

pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
