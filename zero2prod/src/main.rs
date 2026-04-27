use zero2prod::{config::get_config, startup::Application, telemetry};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    // INFO: Telemetry setup
    let subscriber = telemetry::get_subscriber("info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    // INFO: App configuration
    let config = get_config().expect("Failed to read configuration");

    let app = Application::build(config)
        .await
        .expect("Failed to build application");
    app.run_until_stopped().await?;

    Ok(())
}
