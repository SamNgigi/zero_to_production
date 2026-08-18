use axum::response::{IntoResponse, Redirect, Response};
use axum_messages::Messages;

#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Error,
    Warn,
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
        }
    }
}

/// Renders a `FlashError` as a flash message and a 303
#[derive(Debug)]
pub struct FlashRedirect<E> {
    messages: Messages,
    source: E,
}

impl<E: FlashError> IntoResponse for FlashRedirect<E> {
    fn into_response(self) -> Response {
        // Presentation only - logging already happened in `or_flash`
        let location = self.source.redirect_to();
        self.messages.error(self.source.to_string());
        Redirect::to(location).into_response()
    }
}

pub trait FlashResultExt<T, E> {
    // FM for failure mode
    fn or_flash<FM>(self, messages: &Messages) -> Result<T, FlashRedirect<FM>>
    where
        FM: FlashError + From<E>;
}

impl<T, E> FlashResultExt<T, E> for Result<T, E> {
    fn or_flash<FM>(self, messages: &Messages) -> Result<T, FlashRedirect<FM>>
    where
        FM: FlashError + From<E>,
    {
        self.map_err(|e| {
            let source = FM::from(e);
            source.log();
            FlashRedirect {
                messages: messages.clone(),
                source,
            }
        })
    }
}
