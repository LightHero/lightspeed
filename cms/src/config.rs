use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(rename_all = "kebab-case")]
#[clap(setting = clap::AppSettings::AllowExternalSubcommands)]
pub struct CmsConfig {}

impl CmsConfig {
    pub fn build() -> Self {
        Self::parse()
    }
}
