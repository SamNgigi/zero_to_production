use std::fmt::{Debug, Display};
use tokio::task::JoinError;

use zero2prod::{
    config::get_config, newsletter_delivery_worker::run_worker_until_stopped, startup::Application,
    telemetry,
};

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    // INFO: Telemetry setup
    let subscriber = telemetry::get_subscriber("info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    // INFO: App configuration
    let config = get_config().expect("Failed to read configuration");

    let app = Application::build(config.clone())
        .await
        .expect("Failed to build application");
    let application = tokio::spawn(app.run_until_stopped());
    let worker = tokio::spawn(run_worker_until_stopped(config));

    tokio::select! {
        o = application => report_exit("Application task", o),
        o = worker => report_exit("Background worker task", o),
    }

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => tracing::info!("{} has exited.", task_name),
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} task failed.",
                task_name,
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} task failed to complete.",
                task_name
            )
        }
    }
}
