#[derive(serde::Deserialize)]
pub struct Settings {
    pub db: DBSettings,
    pub app_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DBSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub db_name: String,
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

pub fn get_config() -> Result<Settings, config::ConfigError> {
    // Intialize our config reader
    let settings = config::Config::builder()
        // Add config values from a file name 'config.yaml'.
        .add_source(config::File::new("config.yaml", config::FileFormat::Yaml))
        .build()?;
    // Try to convert the read config values into our Setting type
    settings.try_deserialize::<Settings>()
}
