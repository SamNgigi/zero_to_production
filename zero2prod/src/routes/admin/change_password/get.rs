use actix_web::{HttpResponse, http::header::ContentType};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn change_password_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    for msg in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", msg.content())
            .expect("Failed to write msg_html given incoming flash_messages.");
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            include_str!("./change_password.html"),
            msg_html = msg_html
        )))
}
