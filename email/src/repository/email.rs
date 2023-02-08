use crate::config::EmailClientConfig;
use crate::model::email::EmailMessage;
use crate::repository::fixed_recipient_email::FixedRecipientEmailClient;
use crate::repository::full_email::FullEmailClient;
use crate::repository::in_memory_email::InMemoryEmailClient;
use crate::repository::no_ops_email::NoOpsEmailClient;
use lightspeed_core::error::LightSpeedError;
use log::*;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait EmailClient: Send + Sync {
    async fn send(&self, email_message: EmailMessage) -> Result<(), LightSpeedError>;
    fn get_emails(&self) -> Result<Vec<EmailMessage>, LightSpeedError>;
    fn clear_emails(&self) -> Result<(), LightSpeedError>;
    fn retain_emails(&self, retain: Box<dyn FnMut(&EmailMessage) -> bool>) -> Result<(), LightSpeedError>;
}

pub fn new(email_config: EmailClientConfig) -> Result<Arc<dyn EmailClient>, LightSpeedError> {
    let client: Arc<dyn EmailClient> = match &email_config.email_client_type {
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

#[derive(Debug, PartialEq, Copy, Clone, Deserialize)]
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
            _ => Err(LightSpeedError::ConfigurationError { message: format!("Unknown Email client_type [{s}]") }),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[tokio::test]
    async fn should_build_fixed_recipient_client() {
        // Arrange
        let config = EmailClientConfig {
            email_client_type: EmailClientType::InMemory,
            forward_all_emails_to_fixed_recipients: Some(vec!["to@me.com".to_owned()]),
            ..Default::default()
        };

        let mut email = EmailMessage::new();
        email.to.push("original_to@mail.com".to_owned());

        // Act
        let client = new(config).unwrap();
        client.send(email).await.unwrap();

        // Assert
        let emails = client.get_emails().unwrap();
        assert_eq!(1, emails.len());
        let received_email = &emails[0];

        assert_eq!(vec!["to@me.com".to_owned()], received_email.to);
    }

    #[tokio::test]
    async fn should_build_in_memory_client() {
        // Arrange
        let config = EmailClientConfig { email_client_type: EmailClientType::InMemory, ..Default::default() };

        let mut email = EmailMessage::new();
        email.to.push("original_to@mail.com".to_owned());

        // Act
        let client = new(config).unwrap();
        client.send(email).await.unwrap();

        // Assert
        let emails = client.get_emails().unwrap();
        assert_eq!(1, emails.len());
        let received_email = &emails[0];

        assert_eq!(vec!["original_to@mail.com".to_owned()], received_email.to);
    }

    #[test]
    fn should_fail_if_fixed_recipient_empty() {
        // Arrange
        let config = EmailClientConfig {
            email_client_type: EmailClientType::InMemory,
            forward_all_emails_to_fixed_recipients: Some(vec![]),
            ..Default::default()
        };

        let mut email = EmailMessage::new();
        email.to.push("original_to@mail.com".to_owned());

        // Act
        let result = new(config);

        // Assert
        assert!(result.is_err());
    }
}
