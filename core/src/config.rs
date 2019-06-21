use structopt::StructOpt;
use coreutils_logger::config::LoggerConfig;

/// Defines the Logger configuration.
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct CoreConfig {
    #[structopt(flatten)]
    pub logger: LoggerConfig,

    #[structopt(flatten)]
    pub jwt: coreutils_jwt::config::JwtConfig,
}

impl CoreConfig {
    pub fn build() -> Self {
        CoreConfig::from_args()
    }
}

#[cfg(test)]
mod test {}
