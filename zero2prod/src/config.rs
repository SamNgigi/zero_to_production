use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use secrecy::{ExposeSecret, SecretString};

pub enum Environment {
    DEVELOPMENT,
    PRODUCTION,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::DEVELOPMENT => "development",
            Environment::PRODUCTION => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        match val.to_lowercase().as_str() {
            "development" => Ok(Self::DEVELOPMENT),
            "production" => Ok(Self::PRODUCTION),
            other => Err(format!(
                "{} is not a supported environment. \
                    Use either `development` or `production`.",
                other
            )),
        }
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");
    let environment_filename = format!("{}.yaml", environment.as_str());

    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .build()?;
    // Try to convert the read config values into our Setting type
    settings.try_deserialize::<Settings>()
}

#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    pub db: DBSettings,
    pub app: AppSettings,
}

#[derive(Debug, serde::Deserialize)]
pub struct AppSettings {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct DBSettings {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub db_name: String,
    pub max_connections: u32,
}

impl DBSettings {
    pub fn connection_string(&self) -> SecretString {
        SecretString::new(
            format!(
                "postgres://{}:{}@{}:{}/{}",
                self.username,
                self.password.expose_secret(),
                self.host,
                self.port,
                self.db_name
            )
            .into(),
        )
    }

    pub fn connection_string_without_db_name(&self) -> SecretString {
        SecretString::new(
            format!(
                "postgres://{}:{}@{}:{}",
                self.username,
                self.password.expose_secret(),
                self.host,
                self.port
            )
            .into(),
        )
    }
}

pub fn create_pool(cfg: &DBSettings) -> PgPool {
    let options = PgConnectOptions::new()
        .username(&cfg.username)
        .password(cfg.password.expose_secret())
        .port(cfg.port)
        .host(&cfg.host)
        .database(&cfg.db_name);

    PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .connect_lazy_with(options)
}
