use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use chrono::Utc;
// use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberUsername};

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
        subscriber_email=%form.email,
        subscriber_username=%form.username,
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>, // Retrieving a connection from App State
) -> HttpResponse {
    let username = match SubscriberUsername::parse(form.0.username) {
        Ok(username) => username,
        // Return early if the name is invalid, with a 400
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    // `web::Form` is a wrapper around `FormData`
    // `form.0` gives us acces to the underlying `FormData`
    let new_subscriber = NewSubscriber {
        email: form.0.email,
        username,
    };
    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// INFO: `insert_subscriber` takes care of the database logic and it has no awareness of
// the surrounding web framework. Easily portable.
#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, username, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::now_v7(),
        new_subscriber.email,
        new_subscriber.username.as_ref(),
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
