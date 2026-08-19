use axum::{Form, extract::State, response::Redirect};
use axum_messages::Messages;
use secrecy::{ExposeSecret, SecretString};

use crate::{
    authentication::{AuthError, Credentials, UserID, update_password, validate_credentials},
    routes::{AppError, get_username},
    startup::AppState,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    confirm_password: SecretString,
}

#[tracing::instrument(name = "Change Password", skip(state, user_id, messages, form))]
pub async fn change_password(
    State(state): State<AppState>,
    user_id: UserID,
    messages: Messages,
    Form(form): Form<FormData>,
) -> Result<Redirect, AppError> {
    // NOTE: Check new password is not too short
    let new_password_len = form.new_password.expose_secret().len();

    if !(12..129).contains(&new_password_len) {
        messages.error("The New password is too short.");
        return Ok(Redirect::to("/admin/change_password"));
    };

    // NOTE: Check new password and confirm password match
    if form.new_password.expose_secret() != form.confirm_password.expose_secret() {
        messages.error("New password and Confirm password fields DO NOT match. Fields must match.");
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
                messages.error("The Current password is incorrect.");
                return Ok(Redirect::to("/admin/change_password"));
            }
            AuthError::Unexpected(_) => return Err(AppError::Unexpected(e.into())),
            _ => (),
        }
    };

    // NOTE: Change password
    update_password(&state.db_pool, user_id.into_inner(), form.new_password).await?;
    messages.info("You've successfully changed your password.");
    Ok(Redirect::to("/admin/change_password"))
}
