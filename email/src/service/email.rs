use crate::config::EmailConfig;
use crate::model::email::EmailMessage;
use crate::service::full_email::FullEmailService;
use crate::service::in_memory_email::InMemoryEmailService;
use crate::service::no_ops_email::NoOpsEmailService;
use lightspeed_core::error::LightSpeedError;
use std::str::FromStr;

pub trait EmailService {
    fn send(&self, email_message: EmailMessage) -> Result<(), LightSpeedError>;
    fn get_emails(&self) -> Result<Vec<EmailMessage>, LightSpeedError>;
    fn clear_emails(&self) -> Result<(), LightSpeedError>;
}

pub fn new(email_config: EmailConfig) -> Result<Box<EmailService>, LightSpeedError> {
    match &email_config.service_type {
        EmailServiceType::Full => Ok(Box::new(FullEmailService::new(email_config)?)),
        EmailServiceType::InMemory => Ok(Box::new(InMemoryEmailService::new())),
        EmailServiceType::NoOps => Ok(Box::new(NoOpsEmailService::new())),
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EmailServiceType {
    Full,
    InMemory,
    NoOps,
}

impl FromStr for EmailServiceType {
    type Err = LightSpeedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Full" => Ok(EmailServiceType::Full),
            "InMemory" => Ok(EmailServiceType::InMemory),
            "NoOps" => Ok(EmailServiceType::NoOps),
            _ => Err(LightSpeedError::ConfigurationError {
                message: format!("Unknown Email service_type [{}]", s),
            }),
        }
    }
}
