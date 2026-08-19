use anyhow::Context;
use axum::response::{Html, IntoResponse};
use axum_messages::Messages;
use std::fmt::Write;

use crate::routes::AppError;

pub async fn change_password_form(messages: Messages) -> Result<impl IntoResponse, AppError> {
    let mut msg_html = String::new();
    for msg in messages.into_iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", msg.message)
            .map_err(|e| AppError::Unexpected(e.into()))
            .context("Failed to write msg_html given flash message")?
    }
    Ok(Html(format!(
        include_str!("./change_password.html"),
        msg_html = msg_html
    )))
}
