use crate::model::email::EmailMessage;
use crate::service::email::EmailService;
use lightspeed_core::error::LightSpeedError;
use log::warn;

#[derive(Clone, Default)]
pub struct NoOpsEmailService {}

impl NoOpsEmailService {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EmailService for NoOpsEmailService {
    fn send(&self, _email_message: EmailMessage) -> Result<(), LightSpeedError> {
        warn!("NoOpsEmailService.send - Received an email but the email is NOT going to be sent");
        Ok(())
    }

    fn get_emails(&self) -> Result<Vec<EmailMessage>, LightSpeedError> {
        warn!("NoOpsEmailService.get_emails - This is a no ops");

        Ok(vec![])
    }

    fn clear_emails(&self) -> Result<(), LightSpeedError> {
        warn!("NoOpsEmailService.clear_emails - This is a no ops");
        Ok(())
    }
}
