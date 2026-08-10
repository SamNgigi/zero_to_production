use actix_web::{
    HttpResponse, ResponseError,
    http::{
        StatusCode,
        header::{self, HeaderValue},
    },
    web,
};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::PgPool;

use crate::{
    authentication::UserId, domain::SubscriberEmail, email_client::EmailClient, utils::see_other,
};

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

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    txt_content: String,
}

#[tracing::instrument(
    name = "Publish Newsletter.",
    skip(db_pool, email_client, form),
    fields(
        user_id = tracing::field::Empty,
    )
)]
pub async fn publish_newsletter(
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    form: web::Form<FormData>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, PublishError> {
    tracing::Span::current().record("user_id", tracing::field::display(*user_id.into_inner()));

    let subscribers = get_confirmed_subscribers(&db_pool).await?;

    if form.0.title.is_empty() {
        FlashMessage::error("Newsletter issue is missing a title. Issue must have a title.").send();
        return Ok(see_other("/admin/publish_newsletter"));
    }

    if form.0.txt_content.is_empty() {
        FlashMessage::error("Newsletter issue is missing content. Issue must have a content.")
            .send();
        return Ok(see_other("/admin/publish_newsletter"));
    }

    let html_content = get_html(&form.0.txt_content);

    for sub in subscribers {
        match sub {
            Ok(subscriber) => email_client
                .send_email(
                    &subscriber.email,
                    &form.0.title,
                    &html_content,
                    &form.0.txt_content,
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

fn get_html(text: &str) -> String {
    let parser = pulldown_cmark::Parser::new(text);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    ammonia::clean(&html_output)
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
