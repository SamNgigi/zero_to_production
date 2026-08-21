use axum::{Form, extract::State, response::Redirect};
use secrecy::{ExposeSecret, SecretString};

use crate::{
    authentication::{AuthError, Credentials, UserID, update_password, validate_credentials},
    flash::{FlashWriter, Severity},
    routes::{AppError, get_username},
    startup::AppState,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    confirm_password: SecretString,
}

#[tracing::instrument(name = "Change Password", skip(state, user_id, flash_writer, form))]
pub async fn change_password(
    State(state): State<AppState>,
    user_id: UserID,
    flash_writer: FlashWriter,
    Form(form): Form<FormData>,
) -> Result<Redirect, AppError> {
    // NOTE: Check new password is not too short
    let new_password_len = form.new_password.expose_secret().len();

    if !(12..129).contains(&new_password_len) {
        flash_writer.push(Severity::Error, "The New password is too short.");
        return Ok(Redirect::to("/admin/change_password"));
    };

    // NOTE: Check new password and confirm password match
    if form.new_password.expose_secret() != form.confirm_password.expose_secret() {
        flash_writer.push(
            Severity::Error,
            "New password and Confirm password fields DO NOT match. Fields must match.",
        );
        return Ok(Redirect::to("/admin/change_password"));
    }

    // NOTE: Check current password is valid.
    let username = get_username(&state.db_pool, user_id.into_inner()).await?;
    let credentials = Credentials {
        username,
        password: form.current_password,
    };

    if let Err(e) = validate_credentials(&state.db_pool, credentials).await {
        match e {
            AuthError::InvalidPassword(_) => {
                flash_writer.push(Severity::Error, "The Current password is incorrect.");
                return Ok(Redirect::to("/admin/change_password"));
            }
            AuthError::Unexpected(_) => return Err(AppError::Unexpected(e.into())),
            _ => (),
        }
    };

    // NOTE: Change password
    update_password(&state.db_pool, user_id.into_inner(), form.new_password).await?;
    flash_writer.push(Severity::Info, "You've successfully changed your password.");
    Ok(Redirect::to("/admin/change_password"))
}
