pub mod config;
pub mod utils;

use std::str::FromStr;
use thiserror::Error;
use tracing_subscriber::{EnvFilter, fmt::Layer, layer::SubscriberExt};
use tracing::subscriber::set_global_default;
use tracing_appender::non_blocking::WorkerGuard;

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

pub fn setup_logger(logger_config: &config::LoggerConfig) -> Result<Option<WorkerGuard>, LoggerError> {

    if logger_config.stdout_output.stdout_enabled {
        let env_filter = EnvFilter::from_str(&logger_config.env_filter).map_err(|err| {
            LoggerError::LoggerConfigurationError {
                message: format!(
                    "Cannot parse the env_filter: [{}]. err: {}",
                    logger_config.env_filter, err
                ),
            }
        })?;

        let ToDo_fix_HOURLY = 1;
        let file_appender = tracing_appender::rolling::hourly("../target", "prefix.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let subscriber = tracing_subscriber::registry()
            .with(env_filter)
            .with(Layer::new())
            .with(Layer::new().with_ansi(false).with_writer(non_blocking));

        tracing_log::LogTracer::init().map_err(|err| LoggerError::LoggerConfigurationError {
            message: format!("Cannot start the logger LogTracer. err: {}", err),
        })?;
        set_global_default(subscriber).map_err(|err| LoggerError::LoggerConfigurationError {
            message: format!("Cannot start the logger. err: {}", err),
        })?;

        return Ok(Some(guard));
    }

    Ok(None)


}
