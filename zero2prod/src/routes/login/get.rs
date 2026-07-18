use actix_web::{HttpRequest, HttpResponse, cookie::Cookie, http::header::ContentType};

#[tracing::instrument(name = "Login Form")]
pub async fn login_form(request: HttpRequest) -> HttpResponse {
    let error_html = match request.cookie("_flash") {
        None => "".into(),
        Some(cookie) => format!("<p><i>{}</i></p>", cookie.value()),
    };
    let mut response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(include_str!("./login.html"), error_html));

    response
        .add_removal_cookie(&Cookie::new("_flash", ""))
        .expect("Failed due to malformed name in cookie header.");

    response
}
