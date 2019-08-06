use lightspeed_core::error::LightSpeedError;
use crate::config::{EmailConfig};
use crate::model::email::EmailMessage;
use crate::service::email::EmailService;
use log::warn;

#[derive(Clone)]
pub struct NoOpsEmailService {
}

impl NoOpsEmailService {
    pub fn new() -> Self {
        NoOpsEmailService { }
    }
}

impl EmailService for NoOpsEmailService {

    fn send(&self, email_message: EmailMessage) -> Result<(), LightSpeedError> {
        warn!("NoOpsEmailService - Received an email but the email is NOT going to be sent");
        Ok(())
    }

}


