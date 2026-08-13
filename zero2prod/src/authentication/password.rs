use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;
use uuid::Uuid;

use crate::telemetry::spawn_blocking_with_tracing;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid Credentials: No user matching supplied username")]
    UnkownUsername(#[source] anyhow::Error),

    #[error("Invalid Credentials: Invalid password")]
    InvalidPassword(#[source] anyhow::Error),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

pub struct Credentials {
    pub username: String,
    pub password: SecretString,
}

pub async fn validate_credentials(
    db_pool: &PgPool,
    credentials: Credentials,
) -> Result<Uuid, AuthError> {
    let mut _user_id = None;
    let mut expected_password = SecretString::from(
        "$argon2id$v=19$m=19000,t=2,p=1$OqVpaPog6F9sxlWW5VoHkA$4uDo1cl2daKq1ZgmmvtQBfG3wwmI8Nk4i8gHk6pwrYA".to_string()
    );
    if let Some((stored_user_id, stored_password_hash)) =
        get_stored_credentials(db_pool, &credentials.username).await?
    {
        _user_id = Some(stored_user_id);
        expected_password = stored_password_hash;
    };

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task thread")
    .map_err(AuthError::Unexpected)??;

    todo!()
}

#[tracing::instrument(name = "Get Stored Credentials", skip(db_pool, username))]
async fn get_stored_credentials(
    db_pool: &PgPool,
    username: &str,
) -> Result<Option<(Uuid, SecretString)>, anyhow::Error> {
    let row = sqlx::query!(
        r#"
            SELECT user_id, password_hash
                FROM users
            WHERE username = $1
        "#,
        username,
    )
    .fetch_optional(db_pool)
    .await
    .context("Failed to execute SQL query to retrieved stored credentials.")?
    .map(|r| (r.user_id, SecretString::from(r.password_hash)));

    Ok(row)
}

#[tracing::instrument(
    name = "Verify Password Hash",
    skip(expected_password, password_candidate)
)]
fn verify_password_hash(
    expected_password: SecretString,
    password_candidate: SecretString,
) -> Result<(), AuthError> {
    let expected_password_phc_fmt = PasswordHash::new(expected_password.expose_secret())
        .context("Failed to parse password to PHC string format")
        .map_err(AuthError::Unexpected)?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_phc_fmt,
        )
        .context("Invalid Password")
        .map_err(AuthError::InvalidPassword)
}
