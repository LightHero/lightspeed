pub mod config;

use std::str::FromStr;
use thiserror::Error;

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

    log_dispatcher = log_dispatcher
        .level_for("lettre".to_owned(), log::LevelFilter::Warn)
        .level_for("postgres".to_owned(), log::LevelFilter::Warn)
        .level_for("hyper".to_owned(), log::LevelFilter::Warn)
        .level_for("mio".to_owned(), log::LevelFilter::Warn)
        .level_for("tokio_io".to_owned(), log::LevelFilter::Warn)
        .level_for("tokio_reactor".to_owned(), log::LevelFilter::Warn)
        .level_for("tokio_tcp".to_owned(), log::LevelFilter::Warn)
        .level_for("tokio_uds".to_owned(), log::LevelFilter::Warn);

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
