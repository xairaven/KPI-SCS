use crate::config::ConfigError;
use crate::logs::LogError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Configuration. {0}")]
    Config(#[from] ConfigError),

    #[error("Logger setup. {0}")]
    Log(#[from] LogError),
}
