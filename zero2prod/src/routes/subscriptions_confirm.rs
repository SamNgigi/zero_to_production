use axum::{extract::Query, http::StatusCode, response::IntoResponse};

#[derive(serde::Deserialize)]
pub struct Params {
    pub subscription_token: String,
}

#[tracing::instrument(name = "Confirming a new subscriber", skip(params))]
pub async fn confirm(params: Query<Params>) -> impl IntoResponse {
    let _sub_tokens = &params.subscription_token;
    StatusCode::OK
}
