use actix_web::{
    HttpResponse,
    error::InternalError,
    http::{StatusCode, header::LOCATION},
};

pub fn see_other(path: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, path))
        .finish()
}

pub fn e500<E>(e: E) -> InternalError<E>
where
    E: std::fmt::Debug + std::fmt::Display + 'static,
{
    InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR)
}
