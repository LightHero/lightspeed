pub mod config;
pub mod utils;

use std::str::FromStr;
use thiserror::Error;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, EnvFilter};

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

pub fn setup_logger(
    logger_config: &config::LoggerConfig,
) -> Result<Option<WorkerGuard>, LoggerError> {
    if logger_config.stdout_output.stdout_enabled || logger_config.file_output.file_output_enabled {
        let env_filter = EnvFilter::from_str(&logger_config.env_filter).map_err(|err| {
            LoggerError::LoggerConfigurationError {
                message: format!(
                    "Cannot parse the env_filter: [{}]. err: {}",
                    logger_config.env_filter, err
                ),
            }
        })?;

        let subscriber = tracing_subscriber::registry().with(env_filter);

        if logger_config.file_output.file_output_enabled {
            let file_appender = RollingFileAppender::new(
                logger_config
                    .file_output
                    .file_output_rotation
                    .to_tracing_appender_rotation(),
                logger_config.file_output.file_output_directory.to_owned(),
                logger_config.file_output.file_output_name_prefix.to_owned(),
            );

            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            let subscriber =
                subscriber.with(Layer::new().with_ansi(false).with_writer(non_blocking));

            if logger_config.stdout_output.stdout_enabled {
                let subscriber = subscriber.with(
                    Layer::new().with_ansi(logger_config.stdout_output.stdout_use_ansi_colors),
                );
                set_global_logger(subscriber)?;
                return Ok(Some(guard));
            } else {
                set_global_logger(subscriber)?;
                return Ok(Some(guard));
            }
        } else if logger_config.stdout_output.stdout_enabled {
            let subscriber = subscriber
                .with(Layer::new().with_ansi(logger_config.stdout_output.stdout_use_ansi_colors));
            set_global_logger(subscriber)?;
            return Ok(None);
        }
    }
    Ok(None)
}

fn set_global_logger<S>(subscriber: S) -> Result<(), LoggerError>
where
    S: Subscriber + Send + Sync + 'static,
{
    tracing_log::LogTracer::init().map_err(|err| LoggerError::LoggerConfigurationError {
        message: format!("Cannot start the logger LogTracer. err: {}", err),
    })?;
    set_global_default(subscriber).map_err(|err| LoggerError::LoggerConfigurationError {
        message: format!("Cannot start the logger. err: {}", err),
    })
}
