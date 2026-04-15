use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use chrono::Utc;
// use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberUsername};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    username: String,
}

// INFO: We use the TryFrom trait that gives us try_from & try_into that
// takes care of our wire format (url-decoded data collected from a HTML form)
// to our domain model (NewSubscriber)
impl TryFrom<FormData> for NewSubscriber {
    type Error = String;
    fn try_from(val: FormData) -> Result<Self, Self::Error> {
        let username = SubscriberUsername::parse(val.username)?;
        let email = SubscriberEmail::parse(val.email)?;
        Ok(Self { username, email })
    }
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
    let new_subscriber = match form.0.try_into() {
        Ok(form) => form,
        Err(_) => return HttpResponse::BadRequest().finish(),
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
        new_subscriber.email.as_ref(),
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
