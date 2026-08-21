use axum::response::Redirect;

use crate::{
    flash::{FlashWriter, Severity},
    routes::AppError,
    session_state::TypedSession,
};

#[tracing::instrument(name = "Logout", skip(session, flash_writer))]
pub async fn logout(
    session: TypedSession,
    flash_writer: FlashWriter,
) -> Result<Redirect, AppError> {
    session
        .clear()
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?;
    flash_writer.push(Severity::Info, "You've been successfully logged out.");
    Ok(Redirect::to("/login"))
}
