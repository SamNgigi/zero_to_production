use axum::{Form, response::Redirect};
use axum_messages::Messages;
use secrecy::{ExposeSecret, SecretString};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    confirm_password: SecretString,
}

pub async fn change_password(messages: Messages, Form(form): Form<FormData>) -> Redirect {
    if form.new_password.expose_secret() != form.confirm_password.expose_secret() {
        messages.error("New password and Confirm password fields DO NOT match. Fields must match.");
        return Redirect::to("/admin/change_password");
    }
    let _current = form.current_password;
    todo!()
}
