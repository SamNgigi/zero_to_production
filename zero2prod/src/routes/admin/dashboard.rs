use actix_web::{HttpResponse, http::header::ContentType, web};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{authentication::UserId, utils::e500};

pub async fn admin_dashboard(
    db_pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let username = get_username(&db_pool, *user_id.into_inner())
        .await
        .map_err(e500)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            include_str!("./admin_dashboard.html"),
            username = username
        )))
}

pub async fn get_username(db_pool: &PgPool, user_id: Uuid) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
            SELECT username
                FROM users
            WHERE user_id = $1;
        "#,
        user_id
    )
    .fetch_one(db_pool)
    .await
    .context("Failed to execute SQL query to retrieve username.")?;

    Ok(row.username)
}
