use crate::repository::email::EmailClientType;
use lightspeed_core::model::boolean::Boolean;
use clap::Clap;

#[derive(Debug, Clone, Clap)]
#[clap(rename_all = "kebab-case")]
#[clap(setting = clap::AppSettings::AllowExternalSubcommands)]
pub struct EmailClientConfig {
    #[clap(long, env = "LS_EMAIL_CLIENT_TYPE", default_value = "full")]
    pub email_client_type: EmailClientType,

    #[clap(long, env = "LS_EMAIL_CLIENT_TIMEOUT_SECONDS", default_value = "60")]
    pub email_client_timeout_seconds: u64,

    #[clap(long, env = "LS_EMAIL_SERVER_PORT", default_value = "1025")]
    pub email_server_port: u16,

    #[clap(long, env = "LS_EMAIL_SERVER_ADDRESS", default_value = "127.0.0.1")]
    pub email_server_address: String,

    #[clap(long, env = "LS_EMAIL_SERVER_USERNAME", default_value = "")]
    pub email_server_username: String,

    #[clap(long, env = "LS_EMAIL_SERVER_PASSWORD", default_value = "")]
    pub email_server_password: String,

    #[clap(long, env = "LS_EMAIL_SERVER_USE_TLS", default_value = "false")]
    pub email_server_use_tls: Boolean,

    #[clap(long, env = "LS_EMAIL_FORWARD_ALL_EMAILS_TO_FIXED_RECIPIENTS", value_delimiter = ';')]
    pub forward_all_emails_to_fixed_recipients: Option<Vec<String>>,
}

impl EmailClientConfig {
    pub fn build() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_config() {
        let config = EmailClientConfig::build();
        assert!(config.forward_all_emails_to_fixed_recipients.is_none());
    }

    /*
    #[test]
    fn should_build_optional_fixed_recipients() {

        std::env::set_var("LS_EMAIL_FORWARD_ALL_EMAILS_TO_FIXED_RECIPIENTS", "to@me.com;to@they.com");

        let config = EmailClientConfig::build();
        assert_eq!(Some(vec!["to@me.com".to_owned(), "to@they.com".to_owned()]), config.forward_all_emails_to_fixed_recipients);

    }
    */
}
