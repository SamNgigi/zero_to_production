use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Authentication Failed, Invalid username or password.")]
    AuthenticationFailed(#[source] anyhow::Error),

    #[error("The user is not logged in")]
    Unauthenticated(#[source] anyhow::Error),

    #[error("{0}")]
    Validation(String),

    #[error("Something went wrong. Please try again.")]
    Unexpected(#[from] anyhow::Error),
}

/// Present on responses whose error has NOT already been logged
/// Read and logged by TracelLayers's `on_response`.
#[derive(Clone)]
pub struct ErrorContext(pub Arc<AppError>);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let mut response = match &self {
            AppError::AuthenticationFailed(_) => Redirect::to("/login").into_response(),
            AppError::Unauthenticated(_) => Redirect::to("/login").into_response(),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            AppError::Unexpected(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
        };

        response
            .extensions_mut()
            .insert(ErrorContext(Arc::new(self)));
        response
    }
}

// For error response
#[derive(Debug, serde::Serialize)]
pub struct APIErrorBody {
    pub code: &'static str,
    pub msg: String,
}

// For error log reporting
#[derive(Clone)]
pub struct ErrorReport {
    pub message: String, // Display - short, goes on the span for filtering
    pub details: String, // full Debug chain via `error_chain_fmt` - goes on the event
}
