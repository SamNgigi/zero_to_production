use tracing::subscriber::{Subscriber, set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt};

/// Compose multiple layers into a `tracing`'s subscriber
///
/// # Implementation Notes
///
/// We're using `impl Subscriber` as the return type to avoid having to
/// spell out the actual type of the returned subscriber, which is
/// indeed quite complex
/// We nee to explicitly call out that the returned subscriver is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// later on.
pub fn get_subscriber(env_filter: String) -> impl Subscriber + Send + Sync {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = fmt::layer().json().with_current_span(true);

    Registry::default().with(env_filter).with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
