use anyhow::Context;
use axum::{
    Form, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use sqlx::PgPool;

use crate::{
    authentication::UserID,
    domain::SubscriberEmail,
    flash::{FlashWriter, Severity},
    idempotency::{IdempotencyKey, NextAction, save_response, try_processing_response},
    routes::errors::{APIErrorBody, ErrorReport},
    startup::AppState,
};

#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("Something went wrong. Please try again later.")]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for PublishError {
    fn into_response(self) -> Response {
        let report = ErrorReport {
            message: self.to_string(),
            details: match &self {
                PublishError::Unexpected(e) => format!("{e:?}"),
            },
        };

        let (status_code, body) = match &self {
            PublishError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                APIErrorBody {
                    code: "internal_error",
                    msg: "Internal Server Error Occurred".to_string(),
                },
            ),
        };

        let mut response = (status_code, Json(body)).into_response();
        response.extensions_mut().insert(report);
        response
    }
}

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    txt_content: String,
    idempotency_key: String,
}

#[tracing::instrument(name = "Publish newsletter issue", skip(state, form))]
pub async fn publish_newsletter(
    State(state): State<AppState>,
    flash_writer: FlashWriter,
    user_id: UserID,
    Form(form): Form<FormData>,
) -> Result<Response, PublishError> {
    let user_id = user_id.into_inner();
    let subscribers = get_confirmed_subscribers(&state.db_pool).await?;
    let FormData {
        title,
        txt_content,
        idempotency_key,
    } = form;
    let idempotency_key: IdempotencyKey = idempotency_key.try_into()?;

    let transaction =
        match try_processing_response(&state.db_pool, &idempotency_key, user_id).await? {
            NextAction::StartProcessing(t) => t,
            NextAction::ReturnSavedResponse(saved_response) => {
                flash_writer.push(Severity::Info, "Newsletter Issue Published Successfully.");
                return Ok(saved_response);
            }
        };

    let html_content = get_html(&txt_content);
    if title.trim().is_empty() {
        flash_writer.push(
            Severity::Error,
            "Missing title for newsletter issue. Issue must have a title.",
        );
        return Ok(Redirect::to("/admin/publish_newsletter").into_response());
    };
    if txt_content.trim().is_empty() {
        flash_writer.push(
            Severity::Error,
            "Missing content for newsletter issue. Issue must have content.",
        );
        return Ok(Redirect::to("/admin/publish_newsletter").into_response());
    };
    for sub in subscribers {
        match sub {
            Ok(subscriber) => state
                .email_client
                .send_email(&subscriber.email, &title, &html_content, &txt_content)
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

    let response = Redirect::to("/admin/publish_newsletter").into_response();
    let response = save_response(transaction, &idempotency_key, user_id, response).await?;
    flash_writer.push(Severity::Info, "Newsletter Issue Published Successfully.");
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

#[tracing::instrument(
    name = "Get confirmed subscribers"
    skip(db_pool)
)]
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
    .map(|r| match SubscriberEmail::parse(&r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(error) => Err(anyhow::anyhow!(error)),
    })
    .collect();

    Ok(confirmed_subscribers)
}
