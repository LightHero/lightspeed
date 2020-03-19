use crate::config::EmailClientConfig;
use crate::model::email::EmailMessage;
use crate::service::full_email::FullEmailClient;
use crate::service::in_memory_email::InMemoryEmailClient;
use crate::service::no_ops_email::NoOpsEmailClient;
use lightspeed_core::error::LightSpeedError;
use std::str::FromStr;

pub trait EmailClient: Send + Sync {
    fn send(&self, email_message: EmailMessage) -> Result<(), LightSpeedError>;
    fn get_emails(&self) -> Result<Vec<EmailMessage>, LightSpeedError>;
    fn clear_emails(&self) -> Result<(), LightSpeedError>;
    fn retain_emails(
        &self,
        retain: Box<dyn FnMut(&EmailMessage) -> bool>,
    ) -> Result<(), LightSpeedError>;
}

pub fn new(email_config: EmailClientConfig) -> Result<Box<dyn EmailClient>, LightSpeedError> {
    match &email_config.client_type {
        EmailClientType::Full => Ok(Box::new(FullEmailClient::new(email_config)?)),
        EmailClientType::InMemory => Ok(Box::new(InMemoryEmailClient::new())),
        EmailClientType::NoOps => Ok(Box::new(NoOpsEmailClient::new())),
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EmailClientType {
    Full,
    InMemory,
    NoOps,
}

impl FromStr for EmailClientType {
    type Err = LightSpeedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "full" => Ok(EmailClientType::Full),
            "in_memory" => Ok(EmailClientType::InMemory),
            "no_ops" => Ok(EmailClientType::NoOps),
            _ => Err(LightSpeedError::ConfigurationError {
                message: format!("Unknown Email client_type [{}]", s),
            }),
        }
    }
}
