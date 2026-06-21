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
use base64::Engine;
use secrecy::SecretString;
use sqlx::PgPool;

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

#[tracing::instrument(name = "Publish newsletter", skip(db_pool, email_client, body))]
pub async fn publish_newsletter(
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    body: web::Json<BodyData>,
    request: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let _credentials = basic_authentication(request.headers()).map_err(PublishError::Auth)?;
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
    _username: String,
    _password: SecretString,
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
        _username: username,
        _password: SecretString::from(password),
    })
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
