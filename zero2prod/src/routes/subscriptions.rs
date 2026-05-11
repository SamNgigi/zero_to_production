use actix_web::{HttpResponse, web};
use rand::{RngExt, distr::Alphanumeric};
use sqlx::PgPool;

use chrono::Utc;
// use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberUsername},
    email_client::EmailClient,
    startup::ApplicationBaseUrl,
};

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
    skip(form, pool, email_client, base_url),
    fields(
        subscriber_email=%form.email,
        subscriber_username=%form.username,
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>, // Retrieving a connection from App State
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(form) => form,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let subscriber_id = match insert_subscriber(&pool, &new_subscriber).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let subscription_token = generate_subscription_token();

    if store_token(&pool, &subscription_token, subscriber_id)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await
    .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

async fn store_token(
    db_pool: &PgPool,
    subscription_token: &str,
    subscriber_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscription_tokens (subscription_token, subscriber_id)
            VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id,
    )
    .execute(db_pool)
    .await
    .expect("Failed to execute store_token insertion query");
    Ok(())
}

fn generate_subscription_token() -> String {
    std::iter::repeat_with(|| rand::rng().sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Send confirmation email",
    skip(email_client, new_subscriber, base_url)
)]
async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    let text_body = format!(
        "Welcome to our newsletter! Visit {} to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &text_body)
        .await
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
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::now_v7();
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, username, subscribed_at, status)
            VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
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
    Ok(subscriber_id)
}
