use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_messages::Messages;
use secrecy::SecretString;

use crate::{
    authentication::{AuthError, Credentials, validate_credentials},
    startup::AppState,
};

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("Authentication failed. Invalid username or password.")]
    AuthenticationFailed(#[source] anyhow::Error),

    #[error("Something went wrong. Please try again.")]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        match &self {
            LoginError::AuthenticationFailed(_) => {
                (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
            }
            LoginError::Unexpected(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
        }
    }
}

#[derive(serde::Deserialize)]
pub struct LoginFormData {
    username: String,
    password: SecretString,
}

#[tracing::instrument(
    name = " Login Credential Validation",
    skip(state, messages, login_form)
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    State(state): State<AppState>,
    messages: Messages,
    Form(login_form): Form<LoginFormData>,
) -> Redirect {
    let credentials = Credentials {
        username: login_form.username,
        password: login_form.password,
    };
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    match validate_credentials(&state.db_pool, credentials).await {
        Ok(_) => todo!(),
        Err(e) => {
            let e = match e {
                AuthError::Unexpected(_) => LoginError::Unexpected(e.into()),
                AuthError::UnkownUsername(_) | AuthError::InvalidPassword(_) => {
                    LoginError::AuthenticationFailed(e.into())
                }
            };
            tracing::error!(error.cause_chain= ?e, "Login authentication failed.");
            messages.error(e.to_string());
            Redirect::to("/login")
        }
    }
}
