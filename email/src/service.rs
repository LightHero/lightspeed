use crate::model::email::EmailMessage;
use crate::repository::email::EmailClient;
use lightspeed_core::error::LsError;
use log::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct LsEmailService {
    client: Arc<dyn EmailClient>,
}

impl LsEmailService {
    pub fn new(client: Arc<dyn EmailClient>) -> Self {
        Self { client }
    }

    pub async fn send(&self, email_message: EmailMessage) -> Result<(), LsError> {
        debug!("Send email message from [{:?}] to [{:?}]", email_message.from, email_message.to);
        self.client.send(email_message).await
    }

    pub fn client(&self) -> &Arc<dyn EmailClient> {
        &self.client
    }
}
