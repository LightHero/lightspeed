use crate::LoggerError;
use clap::Parser;
use std::str::FromStr;

/// Defines the Logger configuration.
#[derive(Debug, Clone, Parser)]
#[clap(rename_all = "kebab-case")]
pub struct LoggerConfig {
    /// Sets the logger [`EnvFilter`].
    /// Valid values: trace, debug, info, warn, error
    /// Example of a valid filter: "warn,my_crate=info,my_crate::my_mod=debug,[my_span]=trace"
    #[clap(long, env = "LS_LOGGER_LEVEL", default_value = "debug")]
    pub env_filter: String,

    #[clap(flatten)]
    pub stdout_output: StandardOutputConfig,

    #[clap(flatten)]
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

#[derive(Debug, Clone, Parser)]
#[clap(rename_all = "kebab-case")]
pub struct StandardOutputConfig {
    /// Determines whether the Logger should print to standard output.
    /// Valid values: true, false
    #[clap(long, env = "LS_LOGGER_STDOUT_OUTPUT_ENABLED", parse(try_from_str), default_value = "true")]
    pub stdout_enabled: bool,

    #[clap(long, env = "LS_LOGGER_STDOUT_OUTPUT_USE_ANSI_COLORS", parse(try_from_str), default_value = "true")]
    pub stdout_use_ansi_colors: bool,
}

impl Default for StandardOutputConfig {
    fn default() -> Self {
        Self { stdout_use_ansi_colors: true, stdout_enabled: true }
    }
}

#[derive(Debug, Clone, Parser)]
#[clap(rename_all = "kebab-case")]
pub struct FileOutputConfig {
    /// Determines whether the Logger should print to a file.
    /// Valid values: true, false
    #[clap(long, env = "LS_LOGGER_FILE_OUTPUT_ENABLED", parse(try_from_str), default_value = "false")]
    pub file_output_enabled: bool,

    /// The log file location
    #[clap(long, env = "LS_LOGGER_FILE_OUTPUT_DIR", default_value = "/tmp")]
    pub file_output_directory: String,

    /// The log file name's _prefix_
    #[clap(long, env = "LS_LOGGER_FILE_OUTPUT_NAME_PREFIX", default_value = "output.log")]
    pub file_output_name_prefix: String,

    /// The log file rotation strategy
    #[clap(long, env = "LS_LOGGER_FILE_OUTPUT_ROTATION", default_value = "daily")]
    pub file_output_rotation: Rotation,
}

impl Default for FileOutputConfig {
    fn default() -> Self {
        Self {
            file_output_enabled: false,
            file_output_directory: "".to_owned(),
            file_output_name_prefix: "".to_owned(),
            file_output_rotation: Rotation::Daily,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
            _ => Err(LoggerError::LoggerConfigurationError { message: format!("Could not parse rotation [{}]", val) }),
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

impl LoggerConfig {
    pub fn build() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_config() {
        let config = LoggerConfig::build();
        assert!(config.stdout_output.stdout_enabled);
    }
}
