use axum::{Form, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    username: String,
}

/* INFO:
 * `subscribe` orchestrates the work to be done by calling the required
 * routines and translates their outcomes into the proper response
 * according to the rules and conventions of the HTTP protocol
 * */
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(db_pool, form),
    fields(
        subscriber_email = %form.email,
        subscriber_username = %form.username
    )
)]
pub async fn subscribe(
    State(db_pool): State<PgPool>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    match insert_subscriber(&db_pool, &form).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/* INFO:
 * `insert_subscriber` takes care of the database logic and it has no
 * awareness of the surrounding web framework. Easily portable
 * */

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, form)
)]
async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, username, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::now_v7(),
        form.email,
        form.username,
        Utc::now(),
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
        /* INFO:
         * Using the `?` operator to return early
         * if the function failed, returning a sqlx::Error
         * We will talk about error handling in depth later!
         * */
    })?;

    Ok(())
}
