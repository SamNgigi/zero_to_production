use anyhow::Context;
use axum::{Form, extract::State, response::Redirect};
use secrecy::SecretString;

use crate::{
    authentication::{AuthError, Credentials, validate_credentials},
    flash::{FlashError, FlashRedirect, FlashResultExt, FlashWriter, Severity},
    session_state::TypedSession,
    startup::AppState,
};

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("Authentication failed. Invalid username or password.")]
    AuthenticationFailed(#[source] anyhow::Error),

    #[error("Something went wrong. Please try again.")]
    Unexpected(#[from] anyhow::Error),
}

impl From<AuthError> for LoginError {
    fn from(e: AuthError) -> Self {
        match e {
            AuthError::UnknownUsername | AuthError::InvalidPassword(_) => {
                LoginError::AuthenticationFailed(e.into())
            }
            AuthError::Unexpected(_) => LoginError::Unexpected(e.into()),
        }
    }
}

impl FlashError for LoginError {
    fn redirect_to(&self) -> &'static str {
        "/login"
    }
    fn severity(&self) -> Severity {
        match self {
            LoginError::AuthenticationFailed(_) => Severity::Warn,
            LoginError::Unexpected(_) => Severity::Error,
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
    skip(state, flash, session, login_form)
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    State(state): State<AppState>,
    flash: FlashWriter,
    session: TypedSession,
    Form(login_form): Form<LoginFormData>,
) -> Result<Redirect, FlashRedirect<LoginError>> {
    let credentials = Credentials {
        username: login_form.username,
        password: login_form.password,
    };
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    // NOTE: `From<AuthError>` for `LoginError` impl is what buys us the bare `?` here on `validate_credentials`
    // -- It hoists the book's original inline `match` out of the handler body
    let user_id = validate_credentials(&state.db_pool, credentials)
        .await
        .or_flash(&flash)?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    session
        .cycle_id()
        .await
        .context("Failed to rotate the session ID")
        .or_flash(&flash)?;
    session
        .insert_user_id(user_id)
        .await
        .context("Failed to insert user ID in the session")
        .or_flash(&flash)?;

    Ok(Redirect::to("/admin/dashboard"))
}
