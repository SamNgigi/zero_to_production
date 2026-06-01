use actix_web::{HttpResponse, ResponseError, http::StatusCode, web};
use rand::{RngExt, distr::Alphanumeric};
use sqlx::{Executor, PgPool, Postgres, Transaction};

use chrono::Utc;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberUsername},
    email_client::EmailClient,
    startup::ApplicationBaseUrl,
};

//---------------------------------------------
// NOTE: SUBSCRIBE ERROR HANDLING START
//---------------------------------------------
#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    Validation(String),
    #[error("{1}")]
    Unexpected(#[source] Box<dyn std::error::Error>, String),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::Validation(_) => StatusCode::BAD_REQUEST,
            SubscribeError::Unexpected(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

//---------------------------------------------
// NOTE: SUBSCRIBE ERROR HANDLING END
//---------------------------------------------

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
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber = form.0.try_into().map_err(SubscribeError::Validation)?;
    // Instantiating a transaction so that we can make `insert_subscriber` and
    // `store_token` and atomic transaction where both succeed or both fail.
    let mut transaction = pool.begin().await.map_err(|e| {
        SubscribeError::Unexpected(
            Box::new(e),
            "Failed to acquire Postgress Connection from pool.".into(),
        )
    })?;
    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber)
        .await
        .map_err(|e| {
            SubscribeError::Unexpected(
                Box::new(e),
                "Failed to insert new subscriber to database.".into(),
            )
        })?;
    let subscription_token = generate_subscription_token();

    store_token(&mut transaction, &subscription_token, subscriber_id)
        .await
        .map_err(|e| {
            SubscribeError::Unexpected(
                Box::new(e),
                "Failed to store confirmation token to database.".into(),
            )
        })?;
    transaction.commit().await.map_err(|e| {
        SubscribeError::Unexpected(
            Box::new(e),
            "Failed to commit SQL transaction when adding new subscriber to database.".into(),
        )
    })?;

    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await
    .map_err(|e| {
        SubscribeError::Unexpected(
            Box::new(e),
            "Failed to send confirmation email to new subscriber.".into(),
        )
    })?;

    Ok(HttpResponse::Ok().finish())
}

// INFO: `insert_subscriber` takes care of the database logic and it has no awareness of
// the surrounding web framework. Easily portable.
#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction)
)]
pub async fn insert_subscriber(
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
        // INFO: Using the `?` operator to return early
        // if the function failed, returning a sqlx::Error
        // We will talk about error handling in depth later.
    })?;
    Ok(subscriber_id)
}

fn generate_subscription_token() -> String {
    std::iter::repeat_with(|| rand::rng().sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscription_token: &str,
    subscriber_id: Uuid,
) -> Result<(), StoreTokenError> {
    let query = sqlx::query!(
        r#"
            INSERT INTO subscription_tokens (subscription_token, subscriber_id)
            VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id,
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute store_token query: {:?}", e);
        StoreTokenError(e)
    })?;

    Ok(())
}

pub struct StoreTokenError(sqlx::Error);

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered when \
            attempting to store a subscription token. "
        )
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    write!(f, "{}\n\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        write!(f, " Cause by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
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
