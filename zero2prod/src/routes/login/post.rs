use actix_web::{HttpResponse, cookie::Cookie, error::InternalError, http::header::LOCATION, web};
use secrecy::SecretString;
use sqlx::PgPool;

use crate::authentication::{AuthError, Credentials, validate_credentials};

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("Authentication Failed.")]
    AuthenticationFailed(#[source] anyhow::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: SecretString,
}

#[tracing::instrument(
    name = "Login Credential Validation", 
    skip(db_pool, form),
    fields(username = tracing::field::Empty, user_id = tracing::field::Empty),
)]
pub async fn login(
    db_pool: web::Data<PgPool>,
    form: web::Form<FormData>,
) -> Result<HttpResponse, InternalError<LoginError>> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    match validate_credentials(&db_pool, credentials).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", tracing::field::display(&user_id));
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/home"))
                .finish())
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthenticationFailed(e.into()),
                AuthError::Unexpected(_) => LoginError::Unexpected(e.into()),
            };

            let response = HttpResponse::SeeOther()
                .insert_header((LOCATION, "/login"))
                .cookie(Cookie::new("_flash", e.to_string()))
                .finish();

            Err(InternalError::from_response(e, response))
        }
    }
}
