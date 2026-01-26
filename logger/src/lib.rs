pub mod config;
pub mod utils;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::Registry;
use tracing_subscriber::reload::Handle;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{filter::Targets, fmt::Layer, layer::SubscriberExt, reload};

#[derive(Debug)]
pub enum LoggerError {
    LoggerConfigurationError { message: String },
}
impl Display for LoggerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoggerError::LoggerConfigurationError { message } => write!(f, "LoggerConfigurationError: [{message}]"),
        }
    }
}

impl Error for LoggerError {}

impl From<log::SetLoggerError> for LoggerError {
    fn from(error: log::SetLoggerError) -> Self {
        LoggerError::LoggerConfigurationError { message: format!("{error}") }
    }
}

impl From<std::io::Error> for LoggerError {
    fn from(error: std::io::Error) -> Self {
        LoggerError::LoggerConfigurationError { message: format!("{error}") }
    }
}

/// Configure a simple global logger that prints to stdout
pub fn setup_stdout_logger(logger_filter: &str, use_ansi: bool) -> Result<Handle<Targets, Registry>, LoggerError> {
    let env_filter = Targets::from_str(logger_filter).map_err(|err| LoggerError::LoggerConfigurationError {
        message: format!("Cannot parse the env_filter: [{logger_filter}]. err: {err:?}"),
    })?;

    let (env_filter, reload_handle) = reload::Layer::new(env_filter);

    let subscriber = tracing_subscriber::registry().with(env_filter).with(Layer::new().with_ansi(use_ansi));
    set_global_logger(subscriber)?;

    Ok(reload_handle)
}

/// Configure the global logger
pub fn setup_logger(logger_config: &config::LoggerConfig) -> Result<(Handle<Targets, Registry>, Option<WorkerGuard>), LoggerError> {
    let env_filter =
        Targets::from_str(&logger_config.env_filter).map_err(|err| LoggerError::LoggerConfigurationError {
            message: format!("Cannot parse the env_filter: [{}]. err: {:?}", logger_config.env_filter, err),
        })?;
    let (env_filter, reload_handle) = reload::Layer::new(env_filter);

    let (file_subscriber, file_guard) = if logger_config.file_output.file_output_enabled {
        let file_appender = RollingFileAppender::new(
            logger_config.file_output.file_output_rotation.to_tracing_appender_rotation(),
            &logger_config.file_output.file_output_directory,
            &logger_config.file_output.file_output_name_prefix,
        );

        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        (Some(Layer::new().with_ansi(false).with_writer(non_blocking)), Some(guard))
    } else {
        (None, None)
    };

    let stdout_subscriber = if logger_config.stdout_output.stdout_enabled {
        Some(Layer::new().with_ansi(logger_config.stdout_output.stdout_use_ansi_colors))
    } else {
        None
    };

    let subscriber = tracing_subscriber::registry().with(env_filter).with(file_subscriber).with(stdout_subscriber);
    set_global_logger(subscriber)?;

    Ok((reload_handle, file_guard))
}

fn set_global_logger<S>(subscriber: S) -> Result<(), LoggerError>
where
    S: Subscriber + Send + Sync + 'static,
{
    subscriber.try_init().map_err(|err| LoggerError::LoggerConfigurationError {
        message: format!("Cannot start the logger. err: {err:?}"),
    })
}
