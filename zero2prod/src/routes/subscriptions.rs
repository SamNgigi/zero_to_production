use axum::{Form, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    username: String,
}

pub async fn subscribe(
    State(db_pool): State<PgPool>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    let request_id = Uuid::now_v7();
    // INFO: Spans like logs, have an associated level
    // `info_span` creates a span at the info-level
    let request_span = tracing::info_span!(
        "Adding a new subscriber",
        %request_id,
        subscriber_email=%form.email,
        subscriber_username=%form.username
    );

    // FIX: Using `.enter` in an async function is a recipe for disaster!
    // Bear with it now, but don't do that at home!
    // See the following section in the book on 'Instrumenting Futures'
    let _request_span_guard = request_span.enter();

    // INFO: We do not call `.enter` on the query_span!
    // `.instrument` takes care of it at the right moments
    // in the query lifetime
    let query_span = tracing::info_span!("Saving new subscriber detail in the.");
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
    .execute(&db_pool)
    // INFO: First we attach the implementation, then we `.await` it.
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!("request_id {} - New subscriber has been saved.", request_id);
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!(
                "request_id {} - Failed to execute query: {:?}",
                request_id,
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    // INFO:`_request_span_guard` is dropped at the end of `subscribe`
    // That's when we `exit`
}
