use actix_web::{
    HttpResponse,
    error::InternalError,
    http::{
        StatusCode,
        header::{ContentType, LOCATION},
    },
    web,
};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::session_state::TypedSession;

fn e500<E>(e: E) -> InternalError<E>
where
    E: std::fmt::Debug + std::fmt::Display + 'static,
{
    InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn admin_dashboard(
    db_pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    let username = if let Some(user_id) = session.get_user_id().map_err(e500)? {
        get_username(&db_pool, user_id).await.map_err(e500)?
    } else {
        return Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/login"))
            .finish());
    };
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            include_str!("./admin_dashboard.html"),
            username = username
        )))
}

async fn get_username(db_pool: &PgPool, user_id: Uuid) -> Result<String, anyhow::Error> {
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
