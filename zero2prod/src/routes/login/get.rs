use actix_web::{
    HttpRequest, HttpResponse,
    cookie::{Cookie, time::Duration},
    http::header::ContentType,
};

#[tracing::instrument(name = "Login Form")]
pub async fn login_form(request: HttpRequest) -> HttpResponse {
    let error_html = match request.cookie("_flash") {
        None => "".into(),
        Some(cookie) => format!("<p><i>{}</i></p>", cookie.value()),
    };
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .cookie(Cookie::build("_flash", "").max_age(Duration::ZERO).finish())
        .body(format!(include_str!("./login.html"), error_html))
}
