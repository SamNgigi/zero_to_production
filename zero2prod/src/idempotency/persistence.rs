use axum::{
    body::Body,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::Response,
};
use sqlx::PgPool;
use uuid::Uuid;

use super::IdempotencyKey;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

pub async fn get_saved_response(
    db_pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<Option<Response>, anyhow::Error> {
    let row = sqlx::query!(
        r#"
            SELECT  response_status_code as "response_status_code!",
                    response_headers as "response_headers! : Vec<HeaderPairRecord>",
                    response_body as "response_body!"
                FROM idempotency
            WHERE
                idempotency_key = $1 AND 
                user_id = $2;
        "#,
        idempotency_key.as_ref(),
        user_id,
    )
    .fetch_optional(db_pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let status_code = StatusCode::from_u16(row.response_status_code.try_into()?)?;
    let headers = {
        let mut h = HeaderMap::with_capacity(row.response_headers.len());
        for HeaderPairRecord { name, value } in row.response_headers {
            let name = HeaderName::from_bytes(name.as_bytes())?;
            let value = HeaderValue::from_bytes(&value)?;
            h.insert(name, value);
        }
        h
    };

    let mut response = Response::new(Body::from(row.response_body));
    *response.status_mut() = status_code;
    *response.headers_mut() = headers;

    Ok(Some(response))
}
