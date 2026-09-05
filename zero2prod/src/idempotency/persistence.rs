use axum::{
    body::{Body, to_bytes},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::Response,
};
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::IdempotencyKey;

pub async fn try_processing_response(
    db_pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<NextAction, anyhow::Error> {
    let mut transaction = db_pool.begin().await?;
    let query = sqlx::query!(
        r#"
            INSERT INTO idempotency (
                idempotency_key,
                user_id,
                created_at
            )
            VALUES ($1, $2, NOW())
            ON CONFLICT DO NOTHING;
        "#,
        idempotency_key.as_ref(),
        user_id,
    );
    let n_rows_inserted = transaction.execute(query).await?.rows_affected();
    if n_rows_inserted > 0 {
        Ok(NextAction::StartProcessing(transaction))
    } else {
        let saved_response = get_saved_response(db_pool, idempotency_key, user_id)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("Expected to find a saved response, but NONE was found.")
            })?;
        Ok(NextAction::ReturnSavedResponse(saved_response))
    }
}

pub enum NextAction {
    StartProcessing(Transaction<'static, Postgres>),
    ReturnSavedResponse(Response),
}

pub async fn save_response(
    db_pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
    response: Response,
) -> Result<Response, anyhow::Error> {
    // NOTE: Disassembling response into component parts
    let (res_head, res_body) = response.into_parts();
    let status_code = res_head.status.as_u16() as i16;
    let headers = {
        let mut h = Vec::with_capacity(res_head.headers.len());
        for (name, value) in res_head.headers.iter() {
            let name = name.as_str().to_owned();
            let value = value.as_bytes().to_owned();
            h.push(HeaderPairRecord { name, value });
        }
        h
    };
    let body = to_bytes(res_body, usize::MAX).await?;

    // NOTE: Persisting response component parts to DB
    sqlx::query!(
        r#"
            INSERT INTO idempotency (
                idempotency_key,
                user_id,
                response_status_code,
                response_headers,
                response_body,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, NOW())
        "#,
        idempotency_key.as_ref(),
        user_id,
        status_code,
        headers as Vec<HeaderPairRecord>,
        body.as_ref(),
    )
    .execute(db_pool)
    .await?;

    let response = Response::from_parts(res_head, Body::from(body));
    Ok(response)
}

pub async fn get_saved_response(
    db_pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<Option<Response>, anyhow::Error> {
    // NOTE: Fetch response from DB.
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

    // NOTE: Return None if no response was found
    let Some(row) = row else {
        return Ok(None);
    };

    // NOTE: Re-assembling response for retreived component parts
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

    // NOTE: Returning response
    Ok(Some(response))
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}
