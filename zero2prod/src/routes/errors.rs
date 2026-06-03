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

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    write!(f, "{}\n\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        write!(f, " Caused by:\n\t{}", cause)?;
        current = cause.source()
    }
    Ok(())
}
