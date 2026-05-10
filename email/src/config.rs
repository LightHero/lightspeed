use crate::repository::email::EmailClientType;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct EmailClientConfig {
    pub email_client_type: EmailClientType,
    pub email_client_timeout_seconds: u64,
    pub email_server_port: u16,
    pub email_server_address: String,
    pub email_server_username: String,
    pub email_server_password: String,

    /// Opt-out of TLS for the SMTP transport.
    ///
    /// Default is `false`, i.e. TLS is required and the server certificate
    /// is verified. Setting this to `true` switches the transport to lettre's
    /// `builder_dangerous`, which sends credentials and message content in
    /// **plaintext** with no certificate verification — only acceptable
    /// against a local development relay (Mailcrab, MailHog, etc.) on a
    /// trusted network. The flag is named with `dangerous_` to make that
    /// trade-off visible at every call site, and `FullEmailClient::new`
    /// emits a `warn!` whenever this path is taken.
    pub dangerous_no_tls: bool,

    pub forward_all_emails_to_fixed_recipients: Option<Vec<String>>,
}

impl Default for EmailClientConfig {
    fn default() -> Self {
        EmailClientConfig {
            email_client_type: EmailClientType::Full,
            email_client_timeout_seconds: 60,
            email_server_port: 1025,
            email_server_address: "127.0.0.1".to_owned(),
            email_server_username: "".to_owned(),
            email_server_password: "".to_owned(),
            // Secure-by-default. Operators must explicitly opt out for
            // local-only / dev-only setups.
            dangerous_no_tls: false,
            forward_all_emails_to_fixed_recipients: None,
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_config() {
        let config: EmailClientConfig = config::Config::builder().build().unwrap().try_deserialize().unwrap();
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
