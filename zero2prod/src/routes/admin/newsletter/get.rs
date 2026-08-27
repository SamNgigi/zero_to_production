use actix_web::{HttpResponse, http::header::ContentType};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;
use uuid::Uuid;

pub async fn publish_newsletter_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut msg_html = String::new();
    for msg in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", msg.content())
            .expect("Failed to write flash message content to msg_html");
    }
    let idempotency_key = Uuid::now_v7().to_string();
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            include_str!("./newsletter.html"),
            msg_html = msg_html,
            idempotency_key = idempotency_key,
        ))
}
