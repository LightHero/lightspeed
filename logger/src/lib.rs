pub mod config;
pub mod utils;

use std::str::FromStr;
use thiserror::Error;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Error, Debug)]
pub enum LoggerError {
    #[error("LoggerConfigurationError: [{message}]")]
    LoggerConfigurationError { message: String },
}

impl From<log::SetLoggerError> for LoggerError {
    fn from(error: log::SetLoggerError) -> Self {
        LoggerError::LoggerConfigurationError {
            message: format!("{}", error),
        }
    }
}

impl From<std::io::Error> for LoggerError {
    fn from(error: std::io::Error) -> Self {
        LoggerError::LoggerConfigurationError {
            message: format!("{}", error),
        }
    }
}

pub fn setup_logger(logger_config: &config::LoggerConfig) -> Result<(), LoggerError> {
    if logger_config.stdout_output {
        let env_filter = EnvFilter::from_str(&logger_config.env_filter).map_err(|err| {
            LoggerError::LoggerConfigurationError {
                message: format!(
                    "Cannot parse the env_filter: [{}]. err: {}",
                    logger_config.env_filter, err
                ),
            }
        })?;

        FmtSubscriber::builder()
            .with_env_filter(env_filter)
            .try_init()
            .map_err(|err| LoggerError::LoggerConfigurationError {
                message: format!("Cannot start the stdout_output logger. err: {}", err),
            })?;
    }

    Ok(())
}
