use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Params {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm pending subscriber", skip(db_pool, params))]
pub async fn confirm(db_pool: web::Data<PgPool>, params: web::Query<Params>) -> HttpResponse {
    let id = match get_subscriber_id(&db_pool, &params.subscription_token).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    match id {
        None => HttpResponse::BadRequest().finish(),
        Some(subscriber_id) => {
            if confirm_subscriber(&db_pool, subscriber_id).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().finish()
        }
    }
}

#[tracing::instrument(
    name = "Get subscriber_id by the subscription_token",
    skip(db_pool, subscription_token)
)]
async fn get_subscriber_id(
    db_pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
            SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1
        "#,
        subscription_token
    )
    .fetch_optional(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute get_subscriber_id query: {:?}", e);
        e
    })?;
    Ok(result.map(|result| result.subscriber_id))
}

#[tracing::instrument(name = "Confirm pending subscriber")]
async fn confirm_subscriber(db_pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            UPDATE subscriptions
                SET status = 'confirmed'
            WHERE id = $1
        "#,
        subscriber_id,
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute confirm_subscriber query: {:?}", e);
        e
    })?;
    Ok(())
}
