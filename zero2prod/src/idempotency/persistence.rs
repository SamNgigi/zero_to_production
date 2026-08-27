use actix_web::{HttpResponse, http::StatusCode};
use sqlx::PgPool;
use uuid::Uuid;

use crate::idempotency::IdempotencyKey;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

pub async fn get_response(
    db_pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<Option<HttpResponse>, anyhow::Error> {
    let saved_response = sqlx::query!(
        r#"
            SELECT  response_status_code,
                    response_headers as "response_headers: Vec<HeaderPairRecord>",
                    response_body
                FROM idempotency
            WHERE idempotency_key = $1
                AND user_id = $2; 
        "#,
        idempotency_key.as_ref(),
        user_id
    )
    .fetch_optional(db_pool)
    .await?;

    if let Some(row) = saved_response {
        let status_code = StatusCode::from_u16(row.response_status_code.try_into()?)?;
        let mut response = HttpResponse::build(status_code);
        for HeaderPairRecord { name, value } in row.response_headers {
            response.append_header((name, value));
        }
        Ok(Some(response.body(row.response_body)))
    } else {
        Ok(None)
    }
}
