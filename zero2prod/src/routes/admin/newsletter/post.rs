use actix_web::{HttpResponse, web};
use actix_web_flash_messages::FlashMessage;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    authentication::UserId,
    idempotency::{IdempotencyKey, NextAction, save_response, try_processing},
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
    skip(db_pool, form),
    fields(
        user_id = tracing::field::Empty,
    )
)]
pub async fn publish_newsletter(
    db_pool: web::Data<PgPool>,
    form: web::Form<FormData>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = *user_id.into_inner();
    tracing::Span::current().record("user_id", tracing::field::display(user_id));

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
    let mut transaction = match try_processing(&db_pool, &idempotency_key, user_id)
        .await
        .map_err(e500)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            FlashMessage::info("Newsletter Issue Published Successfully.").send();
            return Ok(saved_response);
        }
    };

    let html_content = get_html(&txt_content);
    let newsletter_issue_id =
        insert_newsletter_issue(&mut transaction, &title, &txt_content, &html_content)
            .await
            .map_err(e500)?;
    enqueue_issue_delivery_queue(&mut transaction, newsletter_issue_id)
        .await
        .map_err(e500)?;

    FlashMessage::info("Newsletter Issue Published Successfully.").send();
    let response = see_other("/admin/publish_newsletter");
    let response = save_response(transaction, &idempotency_key, user_id, response)
        .await
        .map_err(e500)?;
    Ok(response)
}

async fn enqueue_issue_delivery_queue(
    transaction: &mut Transaction<'static, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
            INSERT INTO issue_delivery_queue (
                newsletter_issue_id,
                email
            )
            SELECT $1, email
                FROM subscriptions
            WHERE status = 'confirmed';
        "#,
        newsletter_issue_id
    );
    transaction.execute(query).await?;
    Ok(())
}

async fn insert_newsletter_issue(
    transaction: &mut Transaction<'static, Postgres>,
    title: &str,
    txt_content: &str,
    html_content: &str,
) -> Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::now_v7();
    let query = sqlx::query!(
        r#"
            INSERT INTO newsletter_issues (
                newsletter_issue_id,
                title,
                txt_content,
                html_content,
                published_at
            )
            VALUES ($1, $2, $3, $4, NOW())
        "#,
        newsletter_issue_id,
        title,
        txt_content,
        html_content
    );
    transaction.execute(query).await?;
    Ok(newsletter_issue_id)
}

fn get_html(text: &str) -> String {
    let parser = pulldown_cmark::Parser::new(text);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    ammonia::clean(&html_output)
}
