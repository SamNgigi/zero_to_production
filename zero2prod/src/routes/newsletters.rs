use crate::{domain::SubscriberEmail, email_client::EmailClient};
use actix_web::{
    HttpRequest, HttpResponse, ResponseError,
    http::{
        StatusCode,
        header::{self, HeaderMap, HeaderValue},
    },
    web,
};
use anyhow::Context;
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
use base64::Engine;
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("Authentication Failed")]
    Auth(#[source] anyhow::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PublishError::Unexpected(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
            PublishError::Auth(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#)
                    .expect("header value was not a valid UTF8 string.");
                response
                    .headers_mut()
                    .insert(header::WWW_AUTHENTICATE, header_value);
                response
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(Debug, serde::Deserialize)]
pub struct Content {
    html: String,
    plain: String,
}

#[tracing::instrument(
    name = "Publish newsletter",
    skip(db_pool, email_client, body, request),
    fields(
        username = tracing::field::Empty,
        user_id = tracing::field::Empty,
    )
)]
pub async fn publish_newsletter(
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    body: web::Json<BodyData>,
    request: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let credentials = basic_authentication(request.headers()).map_err(PublishError::Auth)?;
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    let user_id = validate_credentials(&db_pool, credentials).await?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    let subscribers = get_confirmed_subscribers(&db_pool).await?;

    for sub in subscribers {
        match sub {
            Ok(subscriber) => email_client
                .send_email(
                    &subscriber.email,
                    &body.title,
                    &body.content.html,
                    &body.content.plain,
                )
                .await
                .with_context(|| {
                    format!("Failed to send newsletter issue to {}", subscriber.email)
                })?,
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping confirmed subscriber. \
                    Store contact details are invalid: {}",
                    error
                );
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}

struct Credentials {
    username: String,
    password: SecretString,
}

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing.")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string.")?;
    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authentication scheme was not 'Basic'.")?;
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' authentication credentails.")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("Decoded credentials was not valid UTF8 string.")?;

    let mut credentials = decoded_credentials.splitn(2, ":");
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth."))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?;

    Ok(Credentials {
        username,
        password: SecretString::from(password),
    })
}

async fn validate_credentials(
    db_pool: &PgPool,
    credentials: Credentials,
) -> Result<Uuid, PublishError> {
    let row = sqlx::query!(
        r#"
            SELECT user_id, password_hash, salt
                FROM users
            WHERE username=$1
        "#,
        credentials.username
    )
    .fetch_optional(db_pool)
    .await
    .context("Failed to perform query to retreive stored credentials.")
    .map_err(PublishError::Unexpected)?;

    let (user_id, expected_password_hash, salt) = match row {
        Some(row) => (row.user_id, row.password_hash, row.salt),
        None => return Err(PublishError::Auth(anyhow::anyhow!("Invalid username."))),
    };

    let hasher = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19_000, 2, 1, None)
            .context("Failed to build argon2 params.")
            .map_err(PublishError::Unexpected)?,
    );

    let salt = argon2::password_hash::SaltString::encode_b64(salt.as_bytes())
        .context("Failed to encode salt to base64.")
        .map_err(PublishError::Unexpected)?;
    let password_hash = hasher
        .hash_password(credentials.password.expose_secret().as_bytes(), &salt)
        .context("Failed to hash password.")
        .map_err(PublishError::Unexpected)?;

    let password_hash = hex::encode(password_hash.hash.expect("Expected hashed password"));

    if expected_password_hash != password_hash {
        Err(PublishError::Auth(anyhow::anyhow!("Invalid password")))
    } else {
        Ok(user_id)
    }
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Get Confirmed Subscriber", skip(db_pool))]
async fn get_confirmed_subscribers(
    db_pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
            SELECT email
                FROM subscriptions
            WHERE status = 'confirmed';
        "#,
    )
    .fetch_all(db_pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(error) => Err(anyhow::anyhow!(error)),
    })
    .collect();
    Ok(confirmed_subscribers)
}
