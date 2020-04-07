use crate::repository::email::EmailClientType;
use lightspeed_core::model::boolean::Boolean;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct EmailClientConfig {
    #[structopt(long, env = "LS_EMAIL_CLIENT_TYPE", default_value = "full")]
    pub client_type: EmailClientType,

    #[structopt(long, env = "LS_EMAIL_SERVER_PORT", default_value = "1025")]
    pub server_port: u16,

    #[structopt(long, env = "LS_EMAIL_SERVER_ADDRESS", default_value = "127.0.0.1")]
    pub server_address: String,

    #[structopt(long, env = "LS_EMAIL_SERVER_USERNAME", default_value = "")]
    pub server_username: String,

    #[structopt(long, env = "LS_EMAIL_SERVER_PASSWORD", default_value = "")]
    pub server_password: String,

    #[structopt(long, env = "LS_EMAIL_SERVER_USE_TLS", default_value = "false")]
    pub server_use_tls: Boolean,
}

impl EmailClientConfig {
    pub fn build() -> Self {
        let app = Self::clap().setting(structopt::clap::AppSettings::AllowExternalSubcommands);
        Self::from_clap(&app.get_matches())
    }
}
