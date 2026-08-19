use axum::{
    extract::{FromRequestParts, Request},
    http::{StatusCode, request::Parts},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_messages::Messages;
use uuid::Uuid;

use crate::{routes::AppError, session_state::TypedSession};

#[derive(Debug, Clone, Copy)]
pub struct UserID(Uuid);

impl UserID {
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl std::fmt::Display for UserID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("UserID missing from request extensions")]
pub struct MissingUserID;

impl IntoResponse for MissingUserID {
    fn into_response(self) -> Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

/// NOTE: The UserID extractor that only reads from the request extensions.
/// All I/O stays in the middleware. Request with UserID rejected and logged
/// with appropriate error type.
impl<S> FromRequestParts<S> for UserID
where
    S: Send + Sync,
{
    type Rejection = MissingUserID;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match parts.extensions.get::<UserID>().copied() {
            Some(user_id) => Ok(user_id),
            None => {
                tracing::error!(
                    http.path = %parts.uri.path(),
                    "A handler requiring authentication is not mounted behind reject_anonymous_user"
                );
                Err(MissingUserID)
            }
        }
    }
}

/// NOTE: Route Guard error type
#[derive(thiserror::Error, Debug)]
pub enum AuthGuardError {
    #[error("User is not logged in.")]
    NotLoggedIn,
    #[error("Something went wrong. Please try again later")]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for AuthGuardError {
    fn into_response(self) -> Response {
        match self {
            AuthGuardError::NotLoggedIn => Redirect::to("/login").into_response(),
            AuthGuardError::Unexpected(e) => AppError::Unexpected(e).into_response(),
        }
    }
}

pub async fn reject_anonymous_user(
    session: TypedSession,
    mut req: Request,
    next: Next,
) -> Result<Response, AuthGuardError> {
    match session.get_user_id().await {
        Ok(Some(user_id)) => {
            req.extensions_mut().insert(UserID(user_id));
            Ok(next.run(req).await)
        }
        Ok(None) => {
            // INFO: We clone the handle out of the extensions rather than extracting `Messages`.
            // `Messages::from_request_parts` calls the crate-private `load()`, which does
            // `messages = take(pending_messages)` — destructive and not idempotent. The
            // handler would then call it a second time and wipe the queue. Pushing needs
            // no load, so we simply never trigger one on this path.
            if let Some(messages) = req.extensions().get::<Messages>().cloned() {
                messages.error(AuthGuardError::NotLoggedIn.to_string());
            };
            Err(AuthGuardError::NotLoggedIn)
        }
        Err(e) => Err(AuthGuardError::Unexpected(
            anyhow::Error::new(e).context("Failed to read the user_id from the session store."),
        )),
    }
}
