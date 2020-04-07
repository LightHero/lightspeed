pub mod config;

use std::str::FromStr;
use thiserror::Error;
use tracing_subscriber::filter::LevelFilter;
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
    let level = LevelFilter::from_str(&logger_config.level).map_err(|err| {
        LoggerError::LoggerConfigurationError {
            message: format!(
                "The specified logger level is not valid: [{}]. err: {}",
                &logger_config.level, err
            ),
        }
    })?;

    let mut filter = EnvFilter::from_default_env()
        // Set the base level when not matched by other directives to WARN.
        .add_directive(level.into());

    if let Some(env_filter) = &logger_config.env_filter {
        // Set the max level for `my_crate::my_mod` to DEBUG, overriding
        // any directives parsed from the env variable.
        filter = filter.add_directive(env_filter.parse().map_err(|err| {
            LoggerError::LoggerConfigurationError {
                message: format!(
                    "Cannot parse the env_filter: [{}]. err: {}",
                    env_filter, err
                ),
            }
        })?);
    }

    if logger_config.stdout_output {
        FmtSubscriber::builder()
            .with_env_filter(filter)
            .try_init()
            .map_err(|err| LoggerError::LoggerConfigurationError {
                message: format!("Cannot start the stdout_output logger. err: {}", err),
            })?;
    }

    Ok(())
}
