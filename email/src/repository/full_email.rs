use crate::config::EmailClientConfig;
use crate::model::email::{EmailAttachment, EmailMessage};
use crate::repository::email::EmailClient;
use lettre::transport::smtp::authentication::{IntoCredentials, Credentials};
use lettre::{AsyncSmtpTransport, Tokio02Transport, Tokio02Connector, Message, Mailbox, SmtpTransport};
use lightspeed_core::error::{ErrorCodes, LightSpeedError};
use log::*;
use std::path::Path;
use std::sync::Arc;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::message::mime::Mime;
use lettre::message::{MultiPartBuilder, Part};

/// A EmailClient implementation that forwards the email to the expected recipients
#[derive(Clone)]
pub struct FullEmailClient {
    email_config: EmailClientConfig,
    client: Arc<AsyncSmtpTransport<Tokio02Connector>>,
}

impl FullEmailClient {
    pub fn new(email_config: EmailClientConfig) -> Result<Self, LightSpeedError> {

        let mut transport_builder = if email_config.server_use_tls.value() {
            let mut transport_builder = AsyncSmtpTransport::<Tokio02Connector>::relay(&email_config.server_address)
                .map_err(|err| LightSpeedError::InternalServerError {
                    message: format!(
                        "FullEmailService.new - Cannot build transport_builder. Err: {}",
                        err
                    ),
                })?;

            transport_builder
        } else {
            AsyncSmtpTransport::<Tokio02Connector>::builder_dangerous(&email_config.server_address)
        };

        transport_builder = transport_builder.port(email_config.server_port);

        if !email_config.server_username.is_empty() && !email_config.server_password.is_empty() {
            let credentials = Credentials::new(
                email_config.server_username.to_owned(),
                email_config.server_password.to_owned(),
            );
            transport_builder = transport_builder.credentials(credentials)
        }


        let smtp_client = transport_builder
            .build();

        Ok(FullEmailClient {
            email_config,
            client: Arc::new(smtp_client),
        })
    }
}

#[async_trait::async_trait]
impl EmailClient for FullEmailClient {
    async fn send(&self, email_message: EmailMessage) -> Result<(), LightSpeedError> {
        let client = self.client.clone();
            debug!("Sending email {:?}", email_message);

            let mut builder = Message::builder();

            if let Some(val) = email_message.subject {
                builder = builder.subject(val)
            }
            if let Some(val) = email_message.from {
                builder = builder.from(parse_mailbox(&val)?)
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

        let ADD_HTML = 'i';
        let ADD_TEXT = 'i';
        /*
                    if let Some(html) = email_message.html {
                        if let Some(text) = email_message.text {
                            builder = builder.alternative(html, text)
                        } else {
                            builder = builder.html(html);
                        }
                    } else if let Some(text) = email_message.text {
                        builder = builder.text(text)
                    }
        */

        let ADD_ATTACHEMT = 'i';
        /*
            for attachment in email_message.attachments {
                match &attachment {
                    EmailAttachment::Binary {
                        body,
                        filename,
                        mime_type,
                    } => {
                        builder = builder
                            .attachment(body, filename, &to_mime_type(mime_type)?)
                            .map_err(|err| LightSpeedError::BadRequest {
                                message: format!(
                                    "Cannot attach the requested attachment [{:?}]. Err: {}",
                                    attachment, err
                                ),
                                code: "",
                            })?;
                    }
                    EmailAttachment::FromFile {
                        path,
                        filename,
                        mime_type,
                    } => {
                        let path = Path::new(path);
                        let filename = filename.as_deref();
                        builder = builder
                            .attachment_from_file(path, filename, &to_mime_type(mime_type)?)
                            .map_err(|err| LightSpeedError::BadRequest {
                                message: format!(
                                    "Cannot attach the requested attachment [{:?}]. Err: {}",
                                    attachment, err
                                ),
                                code: "",
                            })?;
                    }
                }
            }

         */

            let email = builder
                .body("SOME TEST BODY")
                .map_err(|err| LightSpeedError::InternalServerError {
                    message: format!(
                        "FullEmailService.send - Cannot build the email. Err: {}",
                        err
                    ),
                })?;

            let response =
                client
                    .send(email)
                    .await
                    .map_err(|err| LightSpeedError::InternalServerError {
                        message: format!(
                            "FullEmailService.send - Cannot send email to the SMTP server. Err: {:?}",
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
            code: ErrorCodes::PARSE_ERROR,
        })
}

fn to_mime_type(mime_type: &str) -> Result<Mime, LightSpeedError> {
    mime_type
        .parse()
        .map_err(|err| LightSpeedError::BadRequest {
            message: format!("Cannot parse the mime type [{}]. Err: {}", mime_type, err),
            code: "",
        })
}

#[cfg(test)]
pub mod test {

    use super::*;
    use std::str::FromStr;
    use lettre::Address;

    #[test]
    pub fn should_parse_address() {
        assert_eq!(
            Mailbox::new(Some("ufo".to_owned()), Address::from_str("ufo@email.test").unwrap()),
            parse_mailbox("ufo <ufo@email.test>").unwrap()
        );
        assert_eq!(
            Mailbox::new(None, Address::from_str("ufo@email.test").unwrap()),
            parse_mailbox("<ufo@email.test>").unwrap()
        );
        assert_eq!(
            Mailbox::new(None, Address::from_str("ufo@email.test").unwrap()),
            parse_mailbox("ufo@email.test").unwrap()
        );
        assert!(parse_mailbox("ufo").is_err());
    }
}
