use crate::model::email::EmailMessage;
use crate::service::email::EmailService;
use lightspeed_core::error::LightSpeedError;
use log::warn;

#[derive(Clone)]
pub struct NoOpsEmailService {}

impl NoOpsEmailService {
    pub fn new() -> Self {
        NoOpsEmailService {}
    }
}

impl EmailService for NoOpsEmailService {
    fn send(&self, _email_message: EmailMessage) -> Result<(), LightSpeedError> {
        warn!("NoOpsEmailService - Received an email but the email is NOT going to be sent");
        Ok(())
    }
}
