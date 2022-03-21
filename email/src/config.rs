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
    pub email_server_use_tls: bool,
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
            email_server_use_tls: false,
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
