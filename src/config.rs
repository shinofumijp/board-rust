use config::ConfigError;
use dotenv::dotenv;
use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();
        let cfg = config::Config::builder()
            .add_source(config::Environment::default())
            .build()?;
        cfg.try_deserialize()
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    dotenv().expect("Failed to read .env file");
    Config::from_env().expect("Failed to read config")
});
