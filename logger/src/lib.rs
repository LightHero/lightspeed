pub mod config;

use err_derive::Error;
use std::str::FromStr;

#[derive(Error, Debug)]
pub enum LoggerError {
    #[error(display = "LoggerConfigurationError: [{}]", message)]
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
    let mut log_dispatcher = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(
            log::LevelFilter::from_str(&logger_config.level).map_err(|err| {
                LoggerError::LoggerConfigurationError {
                    message: format!(
                        "The specified logger level is not valid: [{}]. err: {}",
                        &logger_config.level, err
                    ),
                }
            })?,
        );
    /*
    .level_for(
        "rust_actix",
        log::LevelFilter::from_str(&logger_config.level).map_err(|err| {
            LoggerError::LoggerConfigurationError {
                message: format!(
                    "The specified logger level is not valid: [{}]. err: {}",
                    &logger_config.level, err
                ),
            }
        })?);
        */

    if logger_config.stdout_output {
        log_dispatcher = log_dispatcher.chain(std::io::stdout());
    }

    if logger_config.stderr_output {
        log_dispatcher = log_dispatcher.chain(std::io::stderr());
    }

    if let Some(path) = &logger_config.file_output_path {
        log_dispatcher = log_dispatcher.chain(fern::log_file(&path)?)
    }

    log_dispatcher.apply()?;

    Ok(())
}
