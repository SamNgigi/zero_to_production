use actix_web::{HttpResponse, body::to_bytes, http::StatusCode};
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::idempotency::IdempotencyKey;

pub enum NextAction {
    StartProcessing(Transaction<'static, Postgres>),
    ReturnSavedResponse(HttpResponse),
}

pub async fn try_processing(
    db_pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<NextAction, anyhow::Error> {
    let mut transaction = db_pool.begin().await?;
    let query = sqlx::query!(
        r#"
            INSERT INTO idempotency (
                user_id,
                idempotency_key,
                created_at
            )
            VALUES ($1, $2, NOW())
            ON CONFLICT DO NOTHING
        "#,
        user_id,
        idempotency_key.as_ref()
    );
    let n_inserted_rows = transaction.execute(query).await?.rows_affected();
    if n_inserted_rows > 0 {
        Ok(NextAction::StartProcessing(transaction))
    } else {
        let saved_response = get_saved_response(db_pool, idempotency_key, user_id)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("Expected to get a saved response, but NONE was found.")
            })?;
        Ok(NextAction::ReturnSavedResponse(saved_response))
    }
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

pub async fn save_response(
    mut transaction: Transaction<'static, Postgres>,
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
    let query = sqlx::query!(
        r#"
            UPDATE idempotency
            SET
                response_status_code = $1,
                response_headers = $2,
                response_body = $3
            WHERE
                idempotency_key = $4 AND
                user_id = $5;

        "#,
        status_code,
        headers as Vec<HeaderPairRecord>,
        body.as_ref(),
        idempotency_key.as_ref(),
        user_id,
    );
    transaction.execute(query).await?;
    transaction.commit().await?;

    // NOTE: Reassemble response & return
    let response = response_head.set_body(body).map_into_boxed_body();
    Ok(response)
}

async fn get_saved_response(
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
