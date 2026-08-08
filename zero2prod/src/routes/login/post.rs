use actix_web::{HttpResponse, error::InternalError, web};
use actix_web_flash_messages::FlashMessage;
use secrecy::SecretString;
use sqlx::PgPool;

use crate::{
    authentication::{AuthError, Credentials, validate_credentials},
    session_state::TypedSession,
    utils::see_other,
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
    skip(db_pool, form, session),
    fields(username = tracing::field::Empty, user_id = tracing::field::Empty),
)]
pub async fn login(
    db_pool: web::Data<PgPool>,
    form: web::Form<FormData>,
    session: TypedSession, // updated param
) -> Result<HttpResponse, InternalError<LoginError>> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    match validate_credentials(&db_pool, credentials).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", tracing::field::display(&user_id));
            session
                .insert_user_id(user_id)
                .map_err(|e| redirect_to_login(LoginError::Unexpected(e.into())))?;
            Ok(see_other("/admin/dashboard"))
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthenticationFailed(e.into()),
                AuthError::Unexpected(_) => LoginError::Unexpected(e.into()),
            };

            Err(redirect_to_login(e))
        }
    }
}

fn redirect_to_login(e: LoginError) -> InternalError<LoginError> {
    FlashMessage::error(e.to_string()).send();
    let response = see_other("/login");
    InternalError::from_response(e, response)
}
