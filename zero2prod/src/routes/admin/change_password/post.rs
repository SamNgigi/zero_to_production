use axum::{Form, extract::State, response::Redirect};
use axum_messages::Messages;
use secrecy::{ExposeSecret, SecretString};

use crate::{
    authentication::{AuthError, Credentials, UserID, validate_credentials},
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
    if form.new_password.expose_secret() != form.confirm_password.expose_secret() {
        messages.error("New password and Confirm password fields DO NOT match. Fields must match.");
        return Ok(Redirect::to("/admin/change_password"));
    }

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

    todo!()
}
