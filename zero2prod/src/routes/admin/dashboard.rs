use anyhow::Context;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{authentication::UserID, routes::AppError, startup::AppState};

pub async fn dashboard(
    State(state): State<AppState>,
    user_id: UserID,
) -> Result<impl IntoResponse, AppError> {
    let username = get_username(&state.db_pool, user_id.into_inner()).await?;
    Ok(Html(format!(
        include_str!("./dashboard.html"),
        username = username
    )))
}

pub async fn get_username(db_pool: &PgPool, user_id: Uuid) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
            SELECT username
                FROM users
            WHERE user_id = $1
        "#,
        user_id,
    )
    .fetch_one(db_pool)
    .await
    .context("Failed to execute SQL query to retrieve username give user_id.")?;

    Ok(row.username)
}
