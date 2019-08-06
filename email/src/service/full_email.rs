use lightspeed_core::error::LightSpeedError;
use crate::config::{EmailConfig};
use crate::model::email::EmailMessage;
use crate::service::email::EmailService;
use lettre::{SmtpClient, Transport, ClientSecurity};
use lettre_email::{Email};
use log::*;

#[derive(Clone)]
pub struct FullEmailService {
    email_config: EmailConfig,
}

impl FullEmailService {
    pub fn new(email_config: EmailConfig) -> Self {
        FullEmailService { email_config }
    }
}

impl EmailService for FullEmailService {

    fn send(&self, email_message: EmailMessage) -> Result<(), LightSpeedError> {

        let mut builder = Email::builder();

        if let Some(val) = email_message.subject {
            builder = builder.subject(val)
        }
        if let Some(val) = email_message.from {
            builder = builder.from(val)
        }
        if let Some(val) = email_message.text {
            builder = builder.text(val)
        }
        if let Some(val) = email_message.html {
            builder = builder.html(val)
        }
        for to in email_message.to {
            builder = builder.to(to)
        }
        for cc in email_message.cc {
            builder = builder.cc(cc)
        }
        for bcc in email_message.bcc {
            builder = builder.bcc(bcc)
        }

        let email = builder.build().map_err(|err| LightSpeedError::InternalServerError {
            message: format!("EmailService.send - Cannot build the email. Err: {}", err)
        })?;

        let smtp_client = SmtpClient::new((self.email_config.server_address.as_str(), self.email_config.server_port), ClientSecurity::None)
            .map_err(|err| LightSpeedError::InternalServerError {
                message: format!("EmailService.send - Cannot connect to the SMTP server. Err: {}", err)
            })?;

        let mut transport = smtp_client.transport();

        let response = transport.send(email.into()).map_err(|err| LightSpeedError::InternalServerError {
            message: format!("EmailService.send - Cannot connect to the SMTP server. Err: {}", err)
        })?;

        debug!("EmailService.send - Email sent. Response code: {}", response.code);
        Ok(())
    }

}


