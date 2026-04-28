use zero2prod::{config::get_config, startup::Application, telemetry};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // INFO: Telemetry setup
    let subscriber = telemetry::get_tracing_subscriber(
        "info".into(),
        std::io::stdout, // sink when app is running
    );
    telemetry::init_tracing_subscriber(subscriber);

    // INFO: App configuration
    let config = get_config().expect("Failed to read configuration");
    tracing::info!("Connecting to database at host: {}", config.db.host);

    let app = Application::build(config)
        .await
        .expect("Failed to build application");

    // INFO: Running App
    app.run_until_stopped()
        .await
        .expect("Failed to run application");

    Ok(())
}
