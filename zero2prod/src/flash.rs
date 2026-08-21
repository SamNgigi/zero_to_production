#![allow(clippy::disallowed_types)]

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Redirect, Response},
};
use axum_messages::{Message, Messages};

const NO_LAYER: (StatusCode, &str) = (
    StatusCode::INTERNAL_SERVER_ERROR,
    "Could not extract flash messages. Is `MessageManagerLayer` installed?",
);

/// NOTE:
/// Write-only handle to flash `Messages` queue.
/// Clones the `Messages` handele straight from the request extensions, so it never triggers
/// destructive and non-idempotent `Messages::load`. Safe to extract any number of times,
/// in any number of layers
#[derive(Debug, Clone)]
pub struct FlashWriter(Messages);

impl FlashWriter {
    pub fn push(&self, severity: Severity, message: impl Into<String>) {
        let queue = self.0.clone();
        match severity {
            Severity::Error => {
                queue.error(message);
            }
            Severity::Warn => {
                queue.warning(message);
            }
            Severity::Info => {
                queue.info(message);
            }
        }
    }
}

// Implement for any generic type `S` that also implements the `Send` + `Sync` traits
// the `FromRequestParts` trait that is generic over `S` for the `FlashWriter` type.
impl<S: Send + Sync> FromRequestParts<S> for FlashWriter {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Messages>()
            .cloned()
            .map(Self)
            .ok_or(NO_LAYER)
    }
}

/// NOTE:
/// Read-only side handler: drains the flash `Messages` queue for rendering.
/// `Messages::from_request_parts` calls the crate-private `load()` which does
/// `messages = take(pending_messages)`. This is the destructive and non idempotent trigger.
/// This extractor memoises itself in the request extensions:
///     At most one `load()` per request regardless of number of callers.
#[derive(Debug, Clone)]
pub struct FlashReader(Messages);

impl<S: Send + Sync> FromRequestParts<S> for FlashReader {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(reader) = parts.extensions.get::<FlashReader>().cloned() {
            return Ok(reader);
        }
        let reader = Self(Messages::from_request_parts(parts, state).await?);
        parts.extensions.insert(reader.clone());
        Ok(reader)
    }
}

impl IntoIterator for FlashReader {
    type Item = Message;
    type IntoIter = Messages;

    fn into_iter(self) -> Self::IntoIter {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Error,
    Warn,
    Info,
}

pub trait FlashError: std::error::Error + Send + Sync + 'static {
    fn redirect_to(&self) -> &'static str;

    fn severity(&self) -> Severity {
        Severity::Error
    }

    /// Emmitted at construction so the handler's span is still current.
    fn log(&self) {
        match self.severity() {
            Severity::Warn => tracing::warn!(
                error = %self, error.cause_chain = ?self, "Request failed"
            ),
            Severity::Error => tracing::error!(
                error = %self, error.cause_chain = ?self, "Request failed"
            ),
            _ => (),
        }
    }
}

/// Renders a `FlashError` as a flash message and a 303
#[derive(Debug)]
pub struct FlashRedirect<E> {
    messages: FlashWriter,
    source: E,
}

impl<E: FlashError> IntoResponse for FlashRedirect<E> {
    fn into_response(self) -> Response {
        // Presentation only - logging already happened in `or_flash`
        let location = self.source.redirect_to();
        self.messages
            .push(self.source.severity(), self.source.to_string());
        Redirect::to(location).into_response()
    }
}

pub trait FlashResultExt<T, E> {
    // FM for failure mode
    fn or_flash<FM>(self, writer: &FlashWriter) -> Result<T, FlashRedirect<FM>>
    where
        FM: FlashError + From<E>;
}

impl<T, E> FlashResultExt<T, E> for Result<T, E> {
    fn or_flash<FM>(self, writer: &FlashWriter) -> Result<T, FlashRedirect<FM>>
    where
        FM: FlashError + From<E>,
    {
        self.map_err(|e| {
            let source = FM::from(e);
            source.log();
            FlashRedirect {
                messages: writer.clone(),
                source,
            }
        })
    }
}
