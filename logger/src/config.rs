use structopt::StructOpt;

/// Defines the Logger configuration.
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct LoggerConfig {

    /// The Logger level
    /// Valid values: trace, debug, info, warn, error
    #[structopt(long, default_value = "info")]
    pub level: String,

    /// Determines whether the Logger should print to standard output.
    /// Valid values: true, false
    #[structopt(long)]
    pub stdout_output: bool,

    /// Determines whether the Logger should print to standard error.
    /// Valid values: true, false
    #[structopt(long)]
    pub stderr_output: bool,

    /// A file path in the file system; if provided, the Logger will append any output to it.
    #[structopt(long)]
    pub file_output_path: Option<String>,

    // #[structopt(short = "o", long = "value_one", default_value = "10000")]
    // pub module_level: HashMap<String, String>,

}

impl Default for LoggerConfig {
    fn default() -> Self {
        LoggerConfig { level: "info".to_owned(), stdout_output: false, stderr_output: false, file_output_path: None }
    }
}