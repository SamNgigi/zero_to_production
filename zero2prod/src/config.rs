use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

#[derive(Debug, serde::Deserialize)]
pub struct AppSettings {
    pub db: DBSettings,
    pub app_port: u16,
}

#[derive(Debug, serde::Deserialize)]
pub struct DBSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub db_name: String,
    pub max_connections: u32,
}

impl DBSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.db_name
        )
    }

    pub fn connection_string_without_db_name(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

pub fn get_config() -> Result<AppSettings, config::ConfigError> {
    // Initializing our config reader
    let settings = config::Config::builder()
        // Add config values from a file named 'config.yaml'
        .add_source(config::File::new("config.yaml", config::FileFormat::Yaml))
        .build()?;
    // Try to convert the read config values into our Setting type
    settings.try_deserialize::<AppSettings>()
}

pub async fn create_pool(cfg: &DBSettings) -> Result<PgPool, sqlx::Error> {
    let options = PgConnectOptions::new()
        .username(&cfg.username)
        .password(&cfg.password)
        .port(cfg.port)
        .host(&cfg.host)
        .database(&cfg.db_name);

    PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .connect_with(options)
        .await
}
