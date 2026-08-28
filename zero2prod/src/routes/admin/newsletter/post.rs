use actix_web::{HttpResponse, web};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::PgPool;

use crate::{
    authentication::UserId,
    domain::SubscriberEmail,
    email_client::EmailClient,
    idempotency::{IdempotencyKey, get_saved_response, save_response},
    utils::{e400, e500, see_other},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    txt_content: String,
    idempotency_key: String,
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
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = *user_id.into_inner();
    tracing::Span::current().record("user_id", tracing::field::display(user_id));

    let subscribers = get_confirmed_subscribers(&db_pool).await.map_err(e500)?;

    let FormData {
        title,
        txt_content,
        idempotency_key,
    } = form.0;

    if title.is_empty() {
        FlashMessage::error("Newsletter issue is missing a title. Issue must have a title.").send();
        return Ok(see_other("/admin/publish_newsletter"));
    }

    if txt_content.is_empty() {
        FlashMessage::error("Newsletter issue is missing content. Issue must have a content.")
            .send();
        return Ok(see_other("/admin/publish_newsletter"));
    }

    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(e400)?;
    // NOTE: Early return if we have a saved response in the database.
    if let Some(response) = get_saved_response(&db_pool, &idempotency_key, user_id)
        .await
        .map_err(e500)?
    {
        FlashMessage::info("Newsletter Issue Published Successfully.").send();
        return Ok(response);
    }

    let html_content = get_html(&txt_content);

    for sub in subscribers {
        match sub {
            Ok(subscriber) => email_client
                .send_email(&subscriber.email, &title, &html_content, &txt_content)
                .await
                .with_context(|| format!("Failed to send newsletter issue to {}", subscriber.email))
                .map_err(e500)?,
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

    FlashMessage::info("Newsletter Issue Published Successfully.").send();
    let response = see_other("/admin/publish_newsletter");
    let response = save_response(&db_pool, &idempotency_key, user_id, response)
        .await
        .map_err(e500)?;
    Ok(response)
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
