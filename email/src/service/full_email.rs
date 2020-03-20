use crate::config::EmailClientConfig;
use crate::model::email::EmailMessage;
use crate::service::email::EmailClient;
use lettre::smtp::authentication::IntoCredentials;
use lettre::smtp::ConnectionReuseParameters;
use lettre::{ClientSecurity, ClientTlsParameters, SmtpClient, SmtpTransport, Transport};
use lettre_email::{Email, Mailbox};
use lightspeed_core::error::LightSpeedError;
use log::*;
use native_tls::TlsConnector;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct FullEmailClient {
    email_config: EmailClientConfig,
    client: Arc<Mutex<SmtpTransport>>,
}

impl FullEmailClient {
    pub fn new(email_config: EmailClientConfig) -> Result<Self, LightSpeedError> {
        let security = if email_config.server_use_tls.value() {
            let tls_builder = TlsConnector::builder();
            let tls_connector =
                tls_builder
                    .build()
                    .map_err(|err| LightSpeedError::InternalServerError {
                        message: format!(
                            "FullEmailService.new - Cannot build TLS connector. Err: {}",
                            err
                        ),
                    })?;
            let tls_parameters =
                ClientTlsParameters::new(email_config.server_address.to_owned(), tls_connector);
            ClientSecurity::Wrapper(tls_parameters)
        } else {
            ClientSecurity::None
        };

        let mut smtp_client = SmtpClient::new(
            (
                email_config.server_address.as_str(),
                email_config.server_port,
            ),
            security,
        )
        .map_err(|err| LightSpeedError::InternalServerError {
            message: format!(
                "FullEmailService.new - Cannot connect to the SMTP server. Err: {}",
                err
            ),
        })?;

        if !email_config.server_username.is_empty() && !email_config.server_password.is_empty() {
            let credentials = (
                email_config.server_username.to_owned(),
                email_config.server_password.to_owned(),
            )
                .into_credentials();
            smtp_client = smtp_client.credentials(credentials)
        }

        let transport = smtp_client
            .connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
            .transport();

        Ok(FullEmailClient {
            email_config,
            client: Arc::new(Mutex::new(transport)),
        })
    }
}

impl EmailClient for FullEmailClient {
    fn send(&self, email_message: EmailMessage) -> Result<(), LightSpeedError> {
        debug!("Sending email {:#?}", email_message);

        let mut builder = Email::builder();

        if let Some(val) = email_message.subject {
            builder = builder.subject(val)
        }
        if let Some(val) = email_message.from {
            builder = builder.from(parse_mailbox(&val)?)
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
            builder = builder.to(parse_mailbox(&to)?)
        }
        for cc in email_message.cc {
            builder = builder.cc(parse_mailbox(&cc)?)
        }
        for bcc in email_message.bcc {
            builder = builder.bcc(parse_mailbox(&bcc)?)
        }

        let email = builder
            .build()
            .map_err(|err| LightSpeedError::InternalServerError {
                message: format!(
                    "FullEmailService.send - Cannot build the email. Err: {}",
                    err
                ),
            })?;

        let mut client = self.client.lock();

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

    fn retain_emails(
        &self,
        _: Box<dyn FnMut(&EmailMessage) -> bool>,
    ) -> Result<(), LightSpeedError> {
        Err(LightSpeedError::InternalServerError {
            message: "FullEmailService.clear_emails - Cannot retain_emails".to_owned(),
        })
    }
}

fn parse_mailbox(address: &str) -> Result<Mailbox, LightSpeedError> {
    address
        .parse::<Mailbox>()
        .map_err(|err| LightSpeedError::BadRequest {
            message: format!("Cannot parse email address [{}]. Err: {}", address, err),
        })
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[test]
    pub fn should_parse_address() {
        assert_eq!(
            Mailbox::new_with_name("ufo".to_owned(), "ufo@email.test".to_owned()),
            parse_mailbox("ufo <ufo@email.test>").unwrap()
        );
        assert_eq!(
            Mailbox::new("ufo@email.test".to_owned()),
            parse_mailbox("<ufo@email.test>").unwrap()
        );
        assert_eq!(
            Mailbox::new("ufo@email.test".to_owned()),
            parse_mailbox("ufo@email.test").unwrap()
        );
        assert!(parse_mailbox("ufo").is_err());
    }
}
