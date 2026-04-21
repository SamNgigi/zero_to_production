use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};

use secrecy::{ExposeSecret, SecretString};
use serde_with::{DisplayFromStr, serde_as};

use crate::domain::SubscriberEmail;

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
        .add_source(config::File::from(configuration_directory.join("local.yaml")).required(false))
        // Add in settings from environment variables (with a prefix of APP and '__' as seperator)
        // E.g. `APP_APPLICATION_PORT=5001` would set `Settings.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;
    // Try to convert the read config values into our Setting type
    settings.try_deserialize::<Settings>()
}

#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    pub db: DBSettings,
    pub app: AppSettings,
    pub email_client: EmailClientSettings,
}

#[derive(Debug, serde::Deserialize)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: SecretString,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
}

#[serde_as]
#[derive(Debug, serde::Deserialize)]
pub struct AppSettings {
    #[serde_as(as = "DisplayFromStr")]
    pub port: u16,
    pub host: String,
}

#[serde_as]
#[derive(Debug, serde::Deserialize, Clone)]
pub struct DBSettings {
    pub username: String,
    pub password: SecretString,
    #[serde_as(as = "DisplayFromStr")]
    pub port: u16,
    pub host: String,
    pub db_name: String,
    pub max_connections: u32,
    // Determine if we demand the connection to be encrypted or not
    pub require_ssl: bool,
}

impl DBSettings {
    pub fn connect_options(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Disable
        };

        PgConnectOptions::new()
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .host(&self.host)
            .ssl_mode(ssl_mode)
            .database(&self.db_name)
    }
}

pub fn create_pool(cfg: &DBSettings) -> PgPool {
    let options = cfg.connect_options();

    PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .connect_lazy_with(options)
}
