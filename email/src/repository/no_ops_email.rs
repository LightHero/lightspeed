use crate::model::email::EmailMessage;
use crate::repository::email::EmailClient;
use lightspeed_core::error::LsError;
use log::warn;

/// A EmailClient implementation that does nothing.
#[derive(Clone, Default)]
pub struct NoOpsEmailClient {}

impl NoOpsEmailClient {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl EmailClient for NoOpsEmailClient {
    async fn send(&self, _email_message: EmailMessage) -> Result<(), LsError> {
        warn!("NoOpsEmailService.send - Received an email but the email is NOT going to be sent");
        Ok(())
    }

    fn get_emails(&self) -> Result<Vec<EmailMessage>, LsError> {
        warn!("NoOpsEmailService.get_emails - This is a no ops");

        Ok(vec![])
    }

    fn clear_emails(&self) -> Result<(), LsError> {
        warn!("NoOpsEmailService.clear_emails - This is a no ops");
        Ok(())
    }

    fn retain_emails(&self, _: Box<dyn FnMut(&EmailMessage) -> bool>) -> Result<(), LsError> {
        warn!("NoOpsEmailService.retain_emails - This is a no ops");
        Ok(())
    }
}
