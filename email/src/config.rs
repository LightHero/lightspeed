use crate::repository::email::EmailClientType;
use lightspeed_core::model::boolean::Boolean;

#[derive(Debug, Clone, Default)]
pub struct EmailClientConfig {
    pub email_client_type: EmailClientType,

    pub email_client_timeout_seconds: u64,

    pub email_server_port: u16,

    pub email_server_address: String,

    pub email_server_username: String,

    pub email_server_password: String,

    pub email_server_use_tls: Boolean,

    pub forward_all_emails_to_fixed_recipients: Option<Vec<String>>,
}
