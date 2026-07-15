use actix_web::{HttpResponse, error::InternalError, http::header::LOCATION, web};
use hmac::{Hmac, KeyInit, Mac};
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;

use crate::{
    authentication::{AuthError, Credentials, validate_credentials},
    startup::HMACSecret,
};

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
    skip(db_pool, form, secret_key),
    fields(username = tracing::field::Empty, user_id = tracing::field::Empty),
)]
pub async fn login(
    db_pool: web::Data<PgPool>,
    form: web::Form<FormData>,
    secret_key: web::Data<HMACSecret>,
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
            let error_query = format!("error={}", urlencoding::Encoded::new(e.to_string()));
            let hmac_tag = {
                type Hmac256 = Hmac<sha2::Sha256>;
                let mut mac = Hmac256::new_from_slice(secret_key.0.expose_secret().as_bytes())
                    .expect("Hmac can take key of any size");
                mac.update(error_query.as_bytes());
                mac.finalize().into_bytes()
            };

            let response = HttpResponse::SeeOther()
                .insert_header((
                    LOCATION,
                    format!("/login?{}&tag={}", error_query, hex::encode(hmac_tag)),
                ))
                .finish();

            Err(InternalError::from_response(e, response))
        }
    }
}
