use structopt::StructOpt;

/// Defines the Logger configuration.
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct LoggerConfig {
    /// The global Logger level.
    /// Valid values: trace, debug, info, warn, error
    #[structopt(long, env = "LS_LOGGER_LEVEL", default_value = "info")]
    pub level: String,

    /// Sets the [`EnvFilter`] that the subscriber will use to determine if
    /// a span or event is enabled.
    /// Example of a valid filter: "my_crate=info,my_crate::my_mod=debug,[my_span]=trace"
    #[structopt(long, env = "LS_LOGGER_ENV_FILTER")]
    pub env_filter: Option<String>,

    /// Determines whether the Logger should print to standard output.
    /// Valid values: true, false
    #[structopt(
        long,
        env = "LS_LOGGER_ENABLE_STDOUT_OUTPUT",
        parse(try_from_str),
        default_value = "true"
    )]
    pub stdout_output: bool,
    // /// Determines whether the Logger should print to standard error.
    // /// Valid values: true, false
    // #[structopt(
    //     long,
    //     env = "LS_LOGGER_ENABLE_STDERR_OUTPUT",
    //     parse(try_from_str),
    //     default_value = "false"
    // )]
    // pub stderr_output: bool,

    // /// A file path in the file system; if provided, the Logger will append any output to it.
    // #[structopt(long, env = "LS_LOGGER_FILE_OUTPUT_PATH")]
    // pub file_output_path: Option<String>,
    // // #[structopt(short = "o", long = "value_one", default_value = "10000")]
    // // pub module_level: HashMap<String, String>,
}

impl LoggerConfig {
    pub fn build() -> Self {
        let app = Self::clap().setting(structopt::clap::AppSettings::AllowExternalSubcommands);
        Self::from_clap(&app.get_matches())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_config() {
        let config = LoggerConfig::build();
        assert!(config.stdout_output);
    }
}
