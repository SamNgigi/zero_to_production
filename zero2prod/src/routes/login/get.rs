use actix_web::{HttpResponse, http::header::ContentType};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

#[tracing::instrument(name = "Login Form", skip(flash_messages))]
pub async fn login_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut msg_html = String::new();
    for msg in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", msg.content())
            .expect("Failed to write msg_html given flash_messages");
    }
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(include_str!("./login.html"), msg_html = msg_html))
}
