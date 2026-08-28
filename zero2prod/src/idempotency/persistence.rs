use actix_web::{HttpResponse, body::to_bytes, http::StatusCode};
use sqlx::PgPool;
use uuid::Uuid;

use crate::idempotency::IdempotencyKey;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

pub async fn save_response(
    db_pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
    http_response: HttpResponse,
) -> Result<HttpResponse, anyhow::Error> {
    // NOTE: Disassemble  response
    let (response_head, response_body) = http_response.into_parts();
    let status_code = response_head.status().as_u16() as i16;
    let headers = {
        let mut h = Vec::with_capacity(response_head.headers().len());
        for (name, value) in response_head.headers().iter() {
            let name = name.to_string().to_owned();
            let value = value.to_owned().as_bytes().to_owned();
            h.push(HeaderPairRecord { name, value });
        }
        h
    };
    let body = to_bytes(response_body)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // NOTE: Insert response
    sqlx::query!(
        r#"
            INSERT INTO idempotency (
                user_id,
                idempotency_key,
                response_status_code,
                response_headers,
                response_body,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
        "#,
        user_id,
        idempotency_key.as_ref(),
        status_code,
        headers as Vec<HeaderPairRecord>, // NOTE: the sqlx.toml file allows us to do this
        body.as_ref(),
    )
    .execute(db_pool)
    .await?;

    // NOTE: Reassemble response & return
    let response = response_head.set_body(body).map_into_boxed_body();
    Ok(response)
}

pub async fn get_saved_response(
    db_pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<Option<HttpResponse>, anyhow::Error> {
    let saved_response = sqlx::query!(
        r#"
            SELECT  response_status_code as "response_status_code!",
                    response_headers as "response_headers!: Vec<HeaderPairRecord>",
                    response_body as "response_body!"
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
