use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::startup::AppState;

#[derive(serde::Deserialize)]
pub struct Params {
    pub subscription_token: String,
}

#[tracing::instrument(name = "Confirming a new subscriber", skip(state, params))]
pub async fn confirm(State(state): State<AppState>, params: Query<Params>) -> impl IntoResponse {
    let id = match get_subscriber_id(&state.db_pool, &params.subscription_token).await {
        Ok(id) => id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    match id {
        None => return StatusCode::INTERNAL_SERVER_ERROR,
        Some(subscriber_id) => {
            if confirm_subscriber(&state.db_pool, subscriber_id)
                .await
                .is_err()
            {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
    }

    StatusCode::OK
}

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
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute get_subscriber_id query: {}", e);
        e
    })?;

    Ok(res.map(|r| r.subscriber_id))
}

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
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute confirm_subscriber: {}", e);
        e
    })?;

    Ok(())
}
