use once_cell::sync::Lazy;
use sqlx::PgPool;

use zero2prod::config::{DBSettings, get_config};

static TRACING: Lazy<()> = Lazy::new(|| todo!());

pub struct TestApp {
    pub _address: String,
    pub _db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let config = get_config().expect("Failed to read configuration");
    let _connection_pool = configure_db(&config.db).await;
    todo!()
}

async fn configure_db(_config: &DBSettings) -> PgPool {
    todo!()
}
