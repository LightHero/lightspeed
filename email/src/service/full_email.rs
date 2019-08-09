use crate::config::EmailConfig;
use crate::model::email::EmailMessage;
use crate::service::email::EmailService;
use lettre::smtp::ConnectionReuseParameters;
use lettre::{ClientSecurity, SmtpClient, SmtpTransport, Transport};
use lettre_email::Email;
use lightspeed_core::error::LightSpeedError;
use log::*;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct FullEmailService {
    email_config: EmailConfig,
    client: Arc<Mutex<SmtpTransport>>,
}

impl FullEmailService {
    pub fn new(email_config: EmailConfig) -> Result<Self, LightSpeedError> {
        let smtp_client = SmtpClient::new(
            (
                email_config.server_address.as_str(),
                email_config.server_port,
            ),
            ClientSecurity::None,
        )
        .map_err(|err| LightSpeedError::InternalServerError {
            message: format!(
                "FullEmailService.new - Cannot connect to the SMTP server. Err: {}",
                err
            ),
        })?;

        let transport = smtp_client
            .connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
            .transport();

        Ok(FullEmailService {
            email_config,
            client: Arc::new(Mutex::new(transport)),
        })
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

        if let Some(html) = email_message.html {
            if let Some(text) = email_message.text {
                builder = builder.alternative(html, text)
            } else {
                builder = builder.html(html);
            }
        } else if let Some(text) = email_message.text {
            builder = builder.text(text)
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

        let email = builder
            .build()
            .map_err(|err| LightSpeedError::InternalServerError {
                message: format!(
                    "FullEmailService.send - Cannot build the email. Err: {}",
                    err
                ),
            })?;

        let mut client =
            self.client
                .lock()
                .map_err(|err| LightSpeedError::InternalServerError {
                    message: format!(
                        "FullEmailService.send - Cannot obtain SMTP client lock. Err: {}",
                        err
                    ),
                })?;

        let response =
            client
                .send(email.into())
                .map_err(|err| LightSpeedError::InternalServerError {
                    message: format!(
                        "FullEmailService.send - Cannot send email to the SMTP server. Err: {}",
                        err
                    ),
                })?;

        debug!(
            "FullEmailService.send - Email sent. Response code: {}",
            response.code
        );
        Ok(())
    }

    fn get_emails(&self) -> Result<Vec<EmailMessage>, LightSpeedError> {
        Err(LightSpeedError::InternalServerError {
            message: "FullEmailService.get_emails - Cannot return sent email".to_owned(),
        })
    }

    fn clear_emails(&self) -> Result<(), LightSpeedError> {
        Err(LightSpeedError::InternalServerError {
            message: "FullEmailService.clear_emails - Cannot clear_emails".to_owned(),
        })
    }
}
