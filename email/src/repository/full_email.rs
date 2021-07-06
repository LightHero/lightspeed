use crate::config::EmailClientConfig;
use crate::model::email::{EmailAttachment, EmailMessage};
use crate::repository::email::EmailClient;
use lettre::message::header::ContentType;
use lettre::message::{Attachment, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use lightspeed_core::error::{ErrorCodes, LightSpeedError};
use log::*;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// A EmailClient implementation that forwards the email to the expected recipients
#[derive(Clone)]
pub struct FullEmailClient {
    email_config: EmailClientConfig,
    client: Arc<SmtpTransport>,
}

impl FullEmailClient {
    pub fn new(email_config: EmailClientConfig) -> Result<Self, LightSpeedError> {
        let mut smtp_transport_builder = if email_config.server_use_tls.value() {
            SmtpTransport::relay(&email_config.server_address).map_err(|err| LightSpeedError::InternalServerError {
                message: format!(
                    "FullEmailService.new - Cannot build SmtpTransport with TLS to the server [{}]. Err: {:?}",
                    email_config.server_address, err
                ),
            })?
        } else {
            SmtpTransport::builder_dangerous(&email_config.server_address)
        };

        smtp_transport_builder = smtp_transport_builder
            .port(email_config.server_port)
            .timeout(Some(Duration::from_secs(email_config.client_timeout_seconds)));

        if !email_config.server_username.is_empty() && !email_config.server_password.is_empty() {
            let credentials =
                Credentials::new(email_config.server_username.to_owned(), email_config.server_password.to_owned());
            smtp_transport_builder = smtp_transport_builder.credentials(credentials);
        }

        let transport = smtp_transport_builder.build();

        Ok(FullEmailClient { email_config, client: Arc::new(transport) })
    }
}

#[async_trait::async_trait]
impl EmailClient for FullEmailClient {
    async fn send(&self, email_message: EmailMessage) -> Result<(), LightSpeedError> {
        let client = self.client.clone();
        tokio::task::spawn_blocking(move || {
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

            let mut multipart = MultiPart::mixed().build();

            if let Some(html) = email_message.html {
                if let Some(text) = email_message.text {
                    multipart = multipart.multipart(MultiPart::alternative_plain_html(text, html));
                } else {
                    multipart = multipart.singlepart(SinglePart::html(html));
                }
            } else if let Some(text) = email_message.text {
                multipart = multipart.singlepart(SinglePart::plain(text));
            };

            for attachment in email_message.attachments {
                match attachment {
                    EmailAttachment::Binary { body, filename, mime_type } => {
                        multipart =
                            multipart.singlepart(Attachment::new(filename).body(body, to_content_type(&mime_type)?));
                    }
                    EmailAttachment::FromFile { path, filename, mime_type } => {
                        let filename = filename.as_deref().unwrap_or_else(|| {
                            Path::new(&path).file_name().and_then(|os_str| os_str.to_str()).unwrap_or("")
                        });

                        let body = std::fs::read(&path).map_err(|err| LightSpeedError::BadRequest {
                            message: format!(
                                "Cannot attach the requested attachment from file [{}]. Err: {:?}",
                                path, err
                            ),
                            code: "",
                        })?;
                        multipart = multipart
                            .singlepart(Attachment::new(filename.to_owned()).body(body, to_content_type(&mime_type)?));
                    }
                }
            }

            let email = builder.multipart(multipart).map_err(|err| LightSpeedError::InternalServerError {
                message: format!("FullEmailService.send - Cannot build the email. Err: {:?}", err),
            })?;

            let response = client.send(&email).map_err(|err| LightSpeedError::InternalServerError {
                message: format!("FullEmailService.send - Cannot send email to the SMTP server. Err: {:?}", err),
            })?;

            debug!("FullEmailService.send - Email sent. Response code: {}", response.code());
            Ok(())
        })
        .await
        .map_err(|err| LightSpeedError::InternalServerError {
            message: format!("FullEmailService.send - Cannot send email to the SMTP server. Err: {:?}", err),
        })?
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

    fn retain_emails(&self, _: Box<dyn FnMut(&EmailMessage) -> bool>) -> Result<(), LightSpeedError> {
        Err(LightSpeedError::InternalServerError {
            message: "FullEmailService.clear_emails - Cannot retain_emails".to_owned(),
        })
    }
}

fn parse_mailbox(address: &str) -> Result<Mailbox, LightSpeedError> {
    address.parse::<Mailbox>().map_err(|err| LightSpeedError::BadRequest {
        message: format!("Cannot parse email address [{}]. Err: {:?}", address, err),
        code: ErrorCodes::PARSE_ERROR,
    })
}

fn to_content_type(mime_type: &str) -> Result<ContentType, LightSpeedError> {
    ContentType::parse(mime_type).map_err(|err| LightSpeedError::BadRequest {
        message: format!("Cannot parse the mime type [{}]. Err: {:?}", mime_type, err),
        code: "",
    })
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[test]
    pub fn should_parse_address() {
        assert_eq!(
            Mailbox::new(Some("ufo".to_owned()), "ufo@email.test".parse().unwrap()),
            parse_mailbox("ufo <ufo@email.test>").unwrap()
        );
        assert_eq!(Mailbox::new(None, "ufo@email.test".parse().unwrap()), parse_mailbox("<ufo@email.test>").unwrap());
        assert_eq!(Mailbox::new(None, "ufo@email.test".parse().unwrap()), parse_mailbox("ufo@email.test").unwrap());
        assert!(parse_mailbox("ufo").is_err());
    }
}
