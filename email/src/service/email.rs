use lightspeed_core::error::LightSpeedError;
use crate::config::{EmailConfig};
use crate::model::email::EmailMessage;
use crate::service::full_email::FullEmailService;
use std::str::FromStr;
use crate::service::no_ops_email::NoOpsEmailService;
use crate::service::in_memory_email::InMemoryEmailService;

pub trait EmailService {
    fn send(&self, email_message: EmailMessage) -> Result<(), LightSpeedError>;
}

pub fn new(email_config: EmailConfig) -> Box<EmailService> {
    match &email_config.service_type {
        EmailServiceType::Full => Box::new(FullEmailService::new(email_config)),
        EmailServiceType::InMemory => Box::new( InMemoryEmailService::new() ),
        EmailServiceType::NoOps => Box::new( NoOpsEmailService::new() )
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EmailServiceType {
    Full,
    InMemory,
    NoOps
}

impl FromStr for EmailServiceType {
    type Err = LightSpeedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Full" => Ok(EmailServiceType::Full),
            "InMemory" => Ok(EmailServiceType::InMemory),
            "NoOps" => Ok(EmailServiceType::NoOps),
            _ => Err(LightSpeedError::ConfigurationError{
                message: format!("Unknown Email service_type [{}]", s)
            }),
        }
    }
}
