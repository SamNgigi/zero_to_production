use axum::{Form, extract::State, response::Response};
use secrecy::SecretString;

use crate::{
    authentication::{Credentials, validate_credentials},
    routes::AppError,
    startup::AppState,
};

#[derive(serde::Deserialize)]
pub struct LoginFormData {
    username: String,
    password: SecretString,
}

#[tracing::instrument(
    name = " Login Credential Validation",
    skip(state, login_form)
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    State(state): State<AppState>,
    Form(login_form): Form<LoginFormData>,
) -> Result<Response, AppError> {
    let credentials = Credentials {
        username: login_form.username,
        password: login_form.password,
    };
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    match validate_credentials(&state.db_pool, credentials).await {
        Ok(_) => todo!(),
        Err(_) => todo!(),
    }
}
