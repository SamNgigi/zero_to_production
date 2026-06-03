use anyhow::Context;
use axum::{
    Form, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use rand::{RngExt, distr::Alphanumeric};
use sqlx::{Executor, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberUsername},
    email_client::EmailClient,
    routes::errors::{APIErrorBody, ErrorReport},
    startup::AppState,
};

#[derive(Debug, thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    Validation(String),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for SubscribeError {
    fn into_response(self) -> Response {
        // {e:?} anyhow's Debug -> numbered chain for unexpected errors
        let report = ErrorReport {
            message: self.to_string(),
            details: match &self {
                SubscribeError::Validation(_) => self.to_string(),
                SubscribeError::Unexpected(e) => format!("{e:?}"),
            },
        };
        let (status, response_body) = match &self {
            SubscribeError::Validation(e) => (
                StatusCode::BAD_REQUEST,
                APIErrorBody {
                    code: "bad_request",
                    msg: e.to_string(),
                },
            ),
            SubscribeError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                APIErrorBody {
                    code: "internal_error",
                    msg: "Internal Server Error Occurred".to_string(),
                },
            ),
        };

        let mut response = (status, Json(response_body)).into_response();
        response.extensions_mut().insert(report);
        response
    }
}

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    username: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let username = SubscriberUsername::parse(form.username)?;
        let email = SubscriberEmail::parse(&form.email)?;

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
) -> Result<StatusCode, SubscribeError> {
    let new_subscriber = NewSubscriber::try_from(form).map_err(SubscribeError::Validation)?;
    let mut transaction = state
        .db_pool
        .begin()
        .await
        .context("Failed to acquire Postgress connection to pool.")?;

    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber)
        .await
        .context("Failed to insert new subscriber to database.")?;

    let subscription_token = generate_subscription_token();

    store_subscription_token(&mut transaction, &subscription_token, subscriber_id)
        .await
        .context("Failed to store confirmation token for new subscriber.")?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction for adding a new subscriber to database.")?;

    send_confirmation_email(
        &state.email_client,
        new_subscriber,
        &state.base_url.0,
        &subscription_token,
    )
    .await
    .context("Failed to send confirmation email to new subscriber.")?;

    Ok(StatusCode::OK)
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
    transaction.execute(query).await?;
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

    transaction.execute(query).await?;

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
        "Welcome to our newsletter!<br/>\
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
