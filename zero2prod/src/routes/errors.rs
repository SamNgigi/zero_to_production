use axum::response::{IntoResponse, Response};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Authentication Failed, Invalid username or password.")]
    AuthenticationFailed(#[source] anyhow::Error),

    #[error("The user is not logged in")]
    Unauthenticated(#[source] anyhow::Error),

    #[error("{0}")]
    Validation(String),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        todo!()
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
