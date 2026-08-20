use anyhow::Context;
use axum::{
    Form, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_messages::Messages;
use sqlx::PgPool;

use crate::{
    domain::SubscriberEmail,
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
}

#[tracing::instrument(name = "Publish newsletter issue", skip(state, form))]
pub async fn publish_newsletter(
    State(state): State<AppState>,
    messages: Messages,
    form: Form<FormData>,
) -> Result<impl IntoResponse, PublishError> {
    let subscribers = get_confirmed_subscribers(&state.db_pool).await?;
    let html_content = get_html(&form.txt_content);
    if form.title.is_empty() {
        messages.error("Missing title for newsletter issue. Issue must have a title.");
        return Ok(Redirect::to("/admin/publish_newsletter"));
    };
    if form.txt_content.is_empty() {
        messages.error("Missing content for newsletter issue. Issue must have content.");
        return Ok(Redirect::to("/admin/publish_newsletter"));
    };
    for sub in subscribers {
        match sub {
            Ok(subscriber) => state
                .email_client
                .send_email(
                    &subscriber.email,
                    &form.title,
                    &html_content,
                    &form.txt_content,
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

    messages.info("Newsletter Issue Published Successfully.");
    Ok(Redirect::to("/admin/publish_newsletter"))
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
