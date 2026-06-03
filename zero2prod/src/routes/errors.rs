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
