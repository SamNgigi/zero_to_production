use tracing::subscriber::{Subscriber, set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::{
    EnvFilter, Registry,
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
};

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
pub fn get_tracing_subscriber<Sink>(env_filter: String, sink: Sink) -> impl Subscriber + Sync + Send
where
    // INFO: This "weird" syntax is a higher-ranked trait bound (HRTB)
    // It basically means that Sink implements the `MakeWriter` trait
    // for all choices of the lifetime parameter `'a`
    // Check out https://doc.rust-lang.org/nomicon/hrtb.html
    // for more details
    Sink: for<'a> MakeWriter<'a> + Sync + Send + 'static,
{
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{},tower_http::trace=debug", env_filter)));
    let formatting_layer = fmt::layer()
        .json()
        .with_writer(sink)
        .with_current_span(true)
        .with_target(true);
    Registry::default().with(env_filter).with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_tracing_subscriber(subscriber: impl Subscriber + Sync + Send) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set tracing subscriber");
}
