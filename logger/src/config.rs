use crate::LoggerError;
use serde::Deserialize;
use std::str::FromStr;

/// Defines the Logger configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LoggerConfig {
    /// Sets the logger [`EnvFilter`].
    /// Valid values: trace, debug, info, warn, error
    /// Example of a valid filter: "warn,my_crate=info,my_crate::my_mod=debug,[my_span]=trace"
    pub env_filter: String,
    pub stdout_output: StandardOutputConfig,
    pub file_output: FileOutputConfig,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            env_filter: "debug".to_owned(),
            stdout_output: StandardOutputConfig::default(),
            file_output: FileOutputConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct StandardOutputConfig {
    /// Determines whether the Logger should print to standard output.
    /// Valid values: true, false
    pub stdout_enabled: bool,
    pub stdout_use_ansi_colors: bool,
}

impl Default for StandardOutputConfig {
    fn default() -> Self {
        Self { stdout_use_ansi_colors: true, stdout_enabled: true }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FileOutputConfig {
    /// Determines whether the Logger should print to a file.
    /// Valid values: true, false
    pub file_output_enabled: bool,

    /// The log file location
    pub file_output_directory: String,

    /// The log file name's _prefix_
    pub file_output_name_prefix: String,

    /// The log file rotation strategy
    pub file_output_rotation: Rotation,
}

impl Default for FileOutputConfig {
    fn default() -> Self {
        Self {
            file_output_enabled: false,
            file_output_directory: ".".to_owned(),
            file_output_name_prefix: "output.log".to_owned(),
            file_output_rotation: Rotation::Never,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Rotation {
    Minutely,
    Hourly,
    Daily,
    Never,
}

impl FromStr for Rotation {
    type Err = LoggerError;

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        match val.to_lowercase().as_ref() {
            "minutely" => Ok(Rotation::Minutely),
            "hourly" => Ok(Rotation::Hourly),
            "daily" => Ok(Rotation::Daily),
            "never" => Ok(Rotation::Never),
            _ => Err(LoggerError::LoggerConfigurationError { message: format!("Could not parse rotation [{val}]") }),
        }
    }
}

impl Rotation {
    pub fn to_tracing_appender_rotation(&self) -> tracing_appender::rolling::Rotation {
        match self {
            Rotation::Minutely => tracing_appender::rolling::Rotation::MINUTELY,
            Rotation::Hourly => tracing_appender::rolling::Rotation::HOURLY,
            Rotation::Daily => tracing_appender::rolling::Rotation::DAILY,
            Rotation::Never => tracing_appender::rolling::Rotation::NEVER,
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_config() {
        let config: LoggerConfig = config::Config::builder().build().unwrap().try_deserialize().unwrap();
        assert!(config.stdout_output.stdout_enabled);
    }
}
