use crate::config::EmailClientConfig;
use crate::model::email::EmailMessage;
use crate::repository::full_email::FullEmailClient;
use crate::repository::in_memory_email::InMemoryEmailClient;
use crate::repository::no_ops_email::NoOpsEmailClient;
use lightspeed_core::error::LightSpeedError;
use std::str::FromStr;
use std::sync::Arc;
use crate::repository::fixed_recipient_email::FixedRecipientEmailClient;
use log::*;

#[async_trait::async_trait]
pub trait EmailClient: Send + Sync {
    async fn send(&self, email_message: EmailMessage) -> Result<(), LightSpeedError>;
    fn get_emails(&self) -> Result<Vec<EmailMessage>, LightSpeedError>;
    fn clear_emails(&self) -> Result<(), LightSpeedError>;
    fn retain_emails(
        &self,
        retain: Box<dyn FnMut(&EmailMessage) -> bool>,
    ) -> Result<(), LightSpeedError>;
}

pub fn new(email_config: EmailClientConfig) -> Result<Arc<dyn EmailClient>, LightSpeedError> {

    let client: Arc<dyn EmailClient> = match &email_config.client_type {
        EmailClientType::Full => Arc::new(FullEmailClient::new(email_config.clone())?),
        EmailClientType::InMemory => Arc::new(InMemoryEmailClient::new()),
        EmailClientType::NoOps => Arc::new(NoOpsEmailClient::new()),
    };

    if let Some(recipients) = email_config.forward_all_emails_to_fixed_recipients {
        warn!("All emails will be sent to the fixed recipients: {}", recipients.join("; "));
        if recipients.is_empty() {
            Err(LightSpeedError::ConfigurationError {
                message: "Cannot build the email client. Based on the current config all emails should be sent to fixed recipients, but the recipient list is empty".to_owned()
            })
        } else {
            Ok(Arc::new(FixedRecipientEmailClient::new(recipients, client)))
        }
    } else {
        Ok(client)
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
