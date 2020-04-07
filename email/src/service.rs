use crate::model::email::EmailMessage;
use crate::repository::email::EmailClient;
use lightspeed_core::error::LightSpeedError;
use std::sync::Arc;

#[derive(Clone)]
pub struct EmailService {
    client: Arc<dyn EmailClient>,
}

impl EmailService {
    pub fn new(client: Arc<dyn EmailClient>) -> Self {
        Self { client }
    }

    pub async fn send(&self, email_message: EmailMessage) -> Result<(), LightSpeedError> {
        self.client.send(email_message).await
    }

    pub fn client(&self) -> &Arc<dyn EmailClient> {
        &self.client
    }
}
