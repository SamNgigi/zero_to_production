use actix_web::{HttpRequest, HttpResponse, Responder};

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

pub async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {name}!")
}
