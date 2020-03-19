use crate::model::email::EmailMessage;
use crate::service::email::EmailClient;
use lightspeed_core::error::LightSpeedError;
use log::warn;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct InMemoryEmailClient {
    emails: Arc<Mutex<Vec<EmailMessage>>>,
}

impl InMemoryEmailClient {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EmailClient for InMemoryEmailClient {
    fn send(&self, email_message: EmailMessage) -> Result<(), LightSpeedError> {
        warn!("InMemoryEmailService - Received an email. The email is NOT going to be sent but kept in memory");

        let mut lock = self
            .emails
            .lock()
            .map_err(|err| LightSpeedError::InternalServerError {
                message: format!(
                    "InMemoryEmailService.send - Cannot obtain lock. Err: [{}]",
                    err
                ),
            })?;

        lock.push(email_message);
        Ok(())
    }

    fn get_emails(&self) -> Result<Vec<EmailMessage>, LightSpeedError> {
        let lock = self
            .emails
            .lock()
            .map_err(|err| LightSpeedError::InternalServerError {
                message: format!(
                    "InMemoryEmailService.clear_emails - Cannot obtain lock . Err: [{}]",
                    err
                ),
            })?;
        Ok(lock.clone())
    }

    fn clear_emails(&self) -> Result<(), LightSpeedError> {
        let mut lock = self
            .emails
            .lock()
            .map_err(|err| LightSpeedError::InternalServerError {
                message: format!(
                    "InMemoryEmailService.clear_emails - Cannot obtain lock . Err: [{}]",
                    err
                ),
            })?;
        lock.clear();
        Ok(())
    }

    fn retain_emails(
        &self,
        mut retain: Box<dyn FnMut(&EmailMessage) -> bool>,
    ) -> Result<(), LightSpeedError> {
        let mut lock = self
            .emails
            .lock()
            .map_err(|err| LightSpeedError::InternalServerError {
                message: format!(
                    "InMemoryEmailService.clear_emails - Cannot obtain lock . Err: [{}]",
                    err
                ),
            })?;
        lock.retain(|email| retain(email));
        Ok(())
    }
}

#[cfg(test)]
pub mod test {

    use super::*;
    use lightspeed_core::utils::new_hyphenated_uuid;

    #[test]
    pub fn should_keep_emails_in_memory() {
        // Arrange
        let mut email_1 = EmailMessage::new();
        email_1.subject = Some(new_hyphenated_uuid());

        let mut email_2 = EmailMessage::new();
        email_2.subject = Some(new_hyphenated_uuid());

        let email_service = InMemoryEmailClient::new();

        // Act
        email_service.send(email_1.clone()).unwrap();
        email_service.send(email_2.clone()).unwrap();
        email_service.send(email_1.clone()).unwrap();

        // Assert
        let emails = email_service.get_emails().unwrap();
        assert_eq!(3, emails.len());
        assert_eq!(email_1.subject, emails[0].subject);
        assert_eq!(email_2.subject, emails[1].subject);
        assert_eq!(email_1.subject, emails[2].subject);
    }

    #[test]
    pub fn should_clear_emails() {
        // Arrange
        let mut email_1 = EmailMessage::new();
        email_1.subject = Some(new_hyphenated_uuid());

        let email_service = InMemoryEmailClient::new();

        // Act
        email_service.send(email_1.clone()).unwrap();
        email_service.send(email_1.clone()).unwrap();
        {
            let emails = email_service.get_emails().unwrap();
            assert!(!emails.is_empty());
        }
        email_service.clear_emails().unwrap();

        // Assert
        let emails = email_service.get_emails().unwrap();
        assert!(emails.is_empty());
    }
}
