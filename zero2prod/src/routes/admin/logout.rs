use axum::response::Redirect;
use axum_messages::Messages;

use crate::{routes::AppError, session_state::TypedSession};

#[tracing::instrument(name = "Logout", skip(session, messages))]
pub async fn logout(session: TypedSession, messages: Messages) -> Result<Redirect, AppError> {
    session
        .clear()
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?;
    messages.info("You've been successfully logged out.");
    Ok(Redirect::to("/login"))
}
