use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use chrono::Utc;
use uuid::Uuid;

use tracing::Instrument;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    username: String,
}

// INFO: `subscribe` orchestrates the work to be done by calling the
// required routines and translates their outcome into the proper response
// according to the rules and conventions of the HTTP protocol
#[tracing::instrument(
    name="Adding a new subscriber"
    skip(form, pool),
    fields(
        request_id = %Uuid::now_v7(),
        subscriber_email=%form.email,
        subscriber_username=%form.username,
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>, // Retrieving a connection from App State
) -> HttpResponse {
    let request_id = Uuid::now_v7();

    let query_span = tracing::info_span!("Saving new subscriber details in the database.");
    match sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, username, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::now_v7(),
        form.email,
        form.username,
        Utc::now(),
    )
    .execute(pool.as_ref())
    // INFO: First we attach the implementation, then we `.await` it
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!("request_id {} - New subscriber has been saved.", request_id);
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!(
                "request_id {} - Failed to execute query: {:?}",
                request_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

// INFO: `insert_subscriber` takes care of the database logic and it has no awareness of
// the surrounding web framework. Easily portable.
#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
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
        // INFO: Using the `?` operator to return early
        // if the function failed, returning a sqlx::Error
        // We will talk about error handling in depth later.
    })?;
    Ok(())
}
