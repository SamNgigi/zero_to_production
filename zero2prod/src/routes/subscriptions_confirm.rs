use anyhow::Context;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    routes::errors::{APIErrorBody, ErrorReport},
    startup::AppState,
};

#[derive(Debug, thiserror::Error)]
pub enum ConfirmError {
    #[error("No subscriber associated with the provided token.")]
    UnknownToken,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for ConfirmError {
    fn into_response(self) -> Response {
        let report = ErrorReport {
            message: self.to_string(),
            details: match &self {
                ConfirmError::UnknownToken => self.to_string(),
                ConfirmError::Unexpected(e) => format!("{e:?}"),
            },
        };

        let (status_code, response_body) = match &self {
            ConfirmError::UnknownToken => (
                StatusCode::BAD_REQUEST,
                APIErrorBody {
                    code: "bad_request",
                    msg: self.to_string(),
                },
            ),
            ConfirmError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                APIErrorBody {
                    code: "internal_error",
                    msg: "Internal Server Error Occurred".to_string(),
                },
            ),
        };

        let mut response = (status_code, Json(response_body)).into_response();
        response.extensions_mut().insert(report);
        response
    }
}

#[derive(serde::Deserialize)]
pub struct Params {
    pub subscription_token: String,
}

#[tracing::instrument(name = "Confirming a new subscriber", skip(state, params))]
pub async fn confirm(
    State(state): State<AppState>,
    params: Query<Params>,
) -> Result<StatusCode, ConfirmError> {
    let id = get_subscriber_id(&state.db_pool, &params.subscription_token)
        .await
        .context("Failed to get subscriber_id with the given subscription_token")?;

    match id {
        None => return Err(ConfirmError::UnknownToken),
        Some(subscriber_id) => confirm_subscriber(&state.db_pool, subscriber_id)
            .await
            .context("Failed to update subscriber to confirmed")?,
    }

    Ok(StatusCode::OK)
}

#[tracing::instrument(
    name = "Get subscriber id given confirmation token",
    skip(db_pool, subscription_token)
)]
async fn get_subscriber_id(
    db_pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let res = sqlx::query!(
        r#"
            SELECT subscriber_id
                FROM subscription_tokens
            WHERE subscription_token = $1
        "#,
        subscription_token
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(res.map(|r| r.subscriber_id))
}

#[tracing::instrument(name = "Update subscriber to confrimed", skip(db_pool, subscriber_id))]
async fn confirm_subscriber(db_pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            UPDATE subscriptions
                SET status = 'confirmed'
            WHERE id = $1
        "#,
        subscriber_id
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
