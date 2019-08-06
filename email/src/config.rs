use structopt::StructOpt;
use crate::service::email::EmailServiceType;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct EmailConfig {

    #[structopt(long, default_value = "Full")]
    pub service_type: EmailServiceType,

    #[structopt(long, default_value = "25")]
    pub server_port: u16,

    #[structopt(long, default_value = "127.0.0.1")]
    pub server_address: String,
}

impl EmailConfig {
    pub fn build() -> Self {
        let app = Self::clap().setting(structopt::clap::AppSettings::AllowExternalSubcommands);
        Self::from_clap(&app.get_matches())
    }
}