use crate::model::email::EmailMessage;
use crate::service::email::EmailService;
use lightspeed_core::error::LightSpeedError;
use log::warn;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct InMemoryEmailService {
    pub emails: Arc<Mutex<Vec<EmailMessage>>>,
}

impl InMemoryEmailService {
    pub fn new() -> Self {
        InMemoryEmailService {
            emails: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl EmailService for InMemoryEmailService {
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
}

impl InMemoryEmailService {
    pub fn emails(&self) -> Arc<Mutex<Vec<EmailMessage>>> {
        self.emails.clone()
    }

    pub fn clear_emails(&self) -> Result<(), LightSpeedError> {
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

        let email_service = InMemoryEmailService::new();

        // Act
        email_service.send(email_1.clone()).unwrap();
        email_service.send(email_2.clone()).unwrap();
        email_service.send(email_1.clone()).unwrap();

        // Assert
        let emails = email_service.emails.lock().unwrap();
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

        let email_service = InMemoryEmailService::new();

        // Act
        email_service.send(email_1.clone()).unwrap();
        email_service.send(email_1.clone()).unwrap();
        {
            let emails = email_service.emails.lock().unwrap();
            assert!(!emails.is_empty());
        }
        email_service.clear_emails().unwrap();

        // Assert
        let emails = email_service.emails.lock().unwrap();
        assert!(emails.is_empty());
    }
}
