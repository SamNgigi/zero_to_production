use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;

use crate::{domain::SubscriberEmail, startup::AppState};

#[derive(serde::Deserialize)]
pub struct BodyData {
    _title: String,
    _content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    _html: String,
    _plain: String,
}

#[tracing::instrument(name = "Publish newsletter issue", skip(state, _body))]
pub async fn publish_newsletter(
    State(state): State<AppState>,
    _body: Json<BodyData>,
) -> impl IntoResponse {
    let subscribers = get_confirmed_subscribers(&state.db_pool).await?;
    StatusCode::OK
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(
    name = "Get confirmed subscribers"
    skip(db_pool)
)]
async fn get_confirmed_subscribers(
    db_pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
            SELECT email
                FROM subscriptions
            WHERE status = 'confirmed';
        "#,
    )
    .fetch_all(db_pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(&r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(error) => Err(anyhow::anyhow!(error)),
    })
    .collect();

    Ok(confirmed_subscribers)
}
