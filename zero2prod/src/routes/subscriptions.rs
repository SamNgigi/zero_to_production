use axum::{Form, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use rand::{RngExt, distr::Alphanumeric};
use sqlx::{Executor, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberUsername},
    email_client::EmailClient,
    startup::AppState,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    username: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let username = SubscriberUsername::parse(form.username)?;
        let email = SubscriberEmail::parse(form.email)?;

        Ok(NewSubscriber { username, email })
    }
}

/* INFO:
 * `subscribe` orchestrates the work to be done by calling the required
 * routines and translates their outcomes into the proper response
 * according to the rules and conventions of the HTTP protocol
 * */
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(state, form),
    fields(
        subscriber_email = %form.email,
        subscriber_username = %form.username
    )
)]
pub async fn subscribe(
    State(state): State<AppState>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    let new_subscriber = match NewSubscriber::try_from(form) {
        Ok(form) => form,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    let mut transaction = match state.db_pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let subscriber_id = match insert_subscriber(&mut transaction, &new_subscriber).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let subscription_token = generate_subscription_token();

    if store_subscription_token(&mut transaction, &subscription_token, subscriber_id)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if transaction.commit().await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if send_confirmation_email(
        &state.email_client,
        new_subscriber,
        &state.base_url.0,
        &subscription_token,
    )
    .await
    .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

/* INFO:
 * `insert_subscriber` takes care of the database logic and it has no
 * awareness of the surrounding web framework. Easily portable
 * */

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(transaction, new_subscriber)
)]
async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::now_v7();
    let query = sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, username, subscribed_at, status)
            VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.username.as_ref(),
        Utc::now(),
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
        /* INFO:
         * Using the `?` operator to return early
         * if the function failed, returning a sqlx::Error
         * We will talk about error handling in depth later!
         * */
    })?;
    Ok(subscriber_id)
}

fn generate_subscription_token() -> String {
    std::iter::repeat_with(|| rand::rng().sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Store subscription_token with new subscriber id",
    skip(transaction, subscription_token, subscriber_id)
)]
async fn store_subscription_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscription_token: &str,
    subscriber_id: Uuid,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
            INSERT INTO subscription_tokens (subscription_token, subscriber_id)
            VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id,
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute store_subscription_token query: {}", e);
        e
    })?;

    Ok(())
}

// INFO: Wrapper for sending a confirmation email
#[tracing::instrument(
    name = "Send new subscriber confirmation email",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token,
    );

    let html_content = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );

    let txt_content = format!(
        "Welcome to our newsletter! Visit {} to confirm your subscription.",
        confirmation_link
    );

    email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            &html_content,
            &txt_content,
        )
        .await?;

    Ok(())
}
