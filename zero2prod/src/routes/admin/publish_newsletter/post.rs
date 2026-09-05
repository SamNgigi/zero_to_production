use axum::{
    Form, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use sqlx::{Executor, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    authentication::UserID,
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
    let FormData {
        title,
        txt_content,
        idempotency_key,
    } = form;
    let idempotency_key: IdempotencyKey = idempotency_key.try_into()?;

    let mut transaction =
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
    let _newsletter_issue_id =
        insert_newsletter_issue(&mut transaction, &title, &txt_content, &html_content)
            .await
            .map_err(|e| PublishError::Unexpected(e.into()))?;
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
            VALUES ($1, $2, $3, $4, NOW());
        "#,
        newsletter_issue_id,
        title,
        txt_content,
        html_content,
    );
    transaction.execute(query).await?;
    Ok(newsletter_issue_id)
}
