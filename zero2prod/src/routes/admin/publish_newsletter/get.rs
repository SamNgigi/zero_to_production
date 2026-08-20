use anyhow::Context;
use axum::response::{Html, IntoResponse};
use axum_messages::Messages;
use std::fmt::Write;

use crate::routes::AppError;

#[tracing::instrument(name = "Publish Newsletter Form", skip(messages))]
pub async fn publish_newsletter_form(messages: Messages) -> Result<impl IntoResponse, AppError> {
    let mut msg_html = String::new();
    for msg in messages.into_iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", msg.message)
            .context("Failed to write msg_html given flash message")?
    }
    Ok(Html(format!(
        include_str!("./publish_newsletter.html"),
        msg_html = msg_html
    )))
}
