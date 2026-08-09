use anyhow::Context;
use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;
use uuid::Uuid;

use crate::telemetry::spawn_blocking_with_tracing;

#[tracing::instrument(name = "Update Password", skip(db_pool, new_password, user_id))]
pub async fn update_password(
    db_pool: &PgPool,
    new_password: SecretString,
    user_id: Uuid,
) -> Result<(), anyhow::Error> {
    let password_hash = spawn_blocking_with_tracing(move || compute_password_hash(new_password))
        .await?
        .context("Failed to compute password_hash for new password.")?;

    sqlx::query!(
        r#"
            UPDATE users
                SET password_hash = $1
            WHERE user_id = $2;
        "#,
        password_hash.expose_secret(),
        user_id,
    )
    .execute(db_pool)
    .await
    .context("Failed to execute SQL to update user's password.")?;

    Ok(())
}

fn compute_password_hash(password: SecretString) -> Result<SecretString, anyhow::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19000, 2, 1, None).expect("Failed to create new Argon2id params."),
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)?
    .to_string();

    Ok(SecretString::from(password_hash))
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid Credentials")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

pub struct Credentials {
    pub username: String,
    pub password: SecretString,
}

#[tracing::instrument(name = "Validate Credentials", skip(db_pool, credentials))]
pub async fn validate_credentials(
    db_pool: &PgPool,
    credentials: Credentials,
) -> Result<Uuid, AuthError> {
    let mut user_id = None;
    let mut expected_password = SecretString::from(
        "$argon2id$v=19$m=19000,t=2,p=1$OqVpaPog6F9sxlWW5VoHkA$4uDo1cl2daKq1ZgmmvtQBfG3wwmI8Nk4i8gHk6pwrYA".to_string()
    );

    if let Some((stored_user_id, stored_expected_password)) =
        get_stored_credentials(db_pool, &credentials.username).await?
    {
        user_id = Some(stored_user_id);
        expected_password = stored_expected_password;
    };

    spawn_blocking_with_tracing(|| verify_password_hash(expected_password, credentials.password))
        .await
        .context("Failed to spawn blocking task thread.")
        .map_err(AuthError::Unexpected)??;

    user_id.ok_or_else(|| AuthError::InvalidCredentials(anyhow::anyhow!("Invalid username")))
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
            WHERE username = $1;
        "#,
        username
    )
    .fetch_optional(db_pool)
    .await
    .context("Failed to execute query to retreive stored credentials")?
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
        .context("Failed to parse password to PHC string format.")
        .map_err(AuthError::Unexpected)?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_phc_fmt,
        )
        .context("Invalid Password")
        .map_err(AuthError::InvalidCredentials)
}
