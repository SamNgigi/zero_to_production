use actix_web::{HttpResponse, http::header::ContentType};
use actix_web_flash_messages::{IncomingFlashMessages, Level};
use std::fmt::Write;

#[tracing::instrument(name = "Login Form", skip(flash_messages))]
pub async fn login_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut error_html = String::new();
    for msg in flash_messages
        .iter()
        .filter(|msg| msg.level() == Level::Error)
    {
        writeln!(error_html, "<p><i>{}</i></p>", msg.content())
            .expect("Failed to write error_html given flash_messages");
    }
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(include_str!("./login.html"), error_html))
}
