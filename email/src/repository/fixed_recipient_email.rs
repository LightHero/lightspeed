use crate::model::email::EmailMessage;
use crate::repository::email::EmailClient;
use lightspeed_core::error::LightSpeedError;
use log::warn;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// A EmailClient implementation that sends all the emails to a fixed recipient ignoring the original
/// recipients of the email.
/// This is useful for development environments or integration testing.
#[derive(Clone)]
pub struct FixedRecipientEmailClient {
    fixed_to_recipients: Vec<String>,
    client: Arc<dyn EmailClient>,
}

impl FixedRecipientEmailClient {
    pub fn new(fixed_to_recipients: Vec<String>, client: Arc<dyn EmailClient>) -> Self {
        Self { fixed_to_recipients, client }
    }
}

impl EmailClient for FixedRecipientEmailClient {
    fn send(
        &self,
        mut email_message: EmailMessage,
    ) -> Pin<Box<dyn Future<Output = Result<(), LightSpeedError>> + Send>> {
        let client = self.client.clone();
        let fixed_to_recipients = self.fixed_to_recipients.clone();

        Box::pin(async move {
            warn!(
                "FixedRecipientEmailClient - Received an email. The email recipients will be substituted by the configured one(s)"
            );

            email_message.subject = Some(to_subject(&email_message.subject.unwrap_or_default(), &email_message.to));

            let original_data_info = to_text(&email_message.to, &email_message.cc, &email_message.bcc);
            if let Some(text) = email_message.text {
                email_message.text = Some(format!("{original_data_info}\n{text}"));
            }

            if let Some(html) = email_message.html {
                email_message.html = Some(format!("<pre>\n{original_data_info}\n</pre>\n</br>\n{html}"));
            }

            if let (None, None) = (&email_message.text, &email_message.html) {
                email_message.text = Some(original_data_info);
            }

            email_message.to = fixed_to_recipients;
            email_message.cc = vec![];
            email_message.bcc = vec![];

            client.send(email_message).await
        })
    }

    fn get_emails(&self) -> Result<Vec<EmailMessage>, LightSpeedError> {
        self.client.get_emails()
    }

    fn clear_emails(&self) -> Result<(), LightSpeedError> {
        self.client.clear_emails()
    }

    fn retain_emails(&self, retain: Box<dyn FnMut(&EmailMessage) -> bool>) -> Result<(), LightSpeedError> {
        self.client.retain_emails(retain)
    }
}

const SECTION_SEPARATOR: &str = "------------------------------------------------------------";
const JOIN_SEPARATOR: &str = "; ";
const RECIPIENT_ALTERATION_MESSAGE: &str =
    "The email recipients were altered programmatically. \nOriginal recipient data:";

fn to_subject(subject: &str, to: &[String]) -> String {
    format!("[TO: {}] {}", to.join(JOIN_SEPARATOR), subject)
}

fn to_text(to: &[String], cc: &[String], bcc: &[String]) -> String {
    let mut text = String::from(SECTION_SEPARATOR);
    text.push('\n');
    text.push_str(RECIPIENT_ALTERATION_MESSAGE);

    text.push_str(&format!("\nTO: {}", to.join(JOIN_SEPARATOR)));
    text.push_str(&format!("\nCC: {}", cc.join(JOIN_SEPARATOR)));
    text.push_str(&format!("\nBCC: {}", bcc.join(JOIN_SEPARATOR)));

    text.push('\n');
    text.push_str(SECTION_SEPARATOR);
    text.push('\n');

    text
}

#[cfg(test)]
pub mod test {

    use super::*;
    use crate::repository::in_memory_email::InMemoryEmailClient;
    use lightspeed_core::utils::new_hyphenated_uuid;

    #[tokio::test]
    async fn should_build_data_from_recipients() {
        // Arrange
        let to = vec![format!("to@email.com"), "two@email.com".to_owned()];
        let cc = vec![format!("cc@email.com"), "two@email.com".to_owned()];
        let bcc = vec![format!("bcc@email.com")];

        // Act
        let info_1 = to_text(&to, &cc, &bcc);

        // Assert
        let expected_info_1 = format!(
            r#"{SECTION_SEPARATOR}
{RECIPIENT_ALTERATION_MESSAGE}
TO: to@email.com; two@email.com
CC: cc@email.com; two@email.com
BCC: bcc@email.com
{SECTION_SEPARATOR}
"#
        );
        assert_eq!(expected_info_1, info_1);
    }

    #[tokio::test]
    async fn should_build_subject_from_recipients() {
        // Arrange
        let to = vec![format!("to@email.com"), "two@email.com".to_owned()];
        let subject = new_hyphenated_uuid();
        // Act
        let info_1 = to_subject(&subject, &to);

        // Assert
        assert_eq!(format!("[TO: to@email.com; two@email.com] {subject}"), info_1);
    }

    #[tokio::test]
    async fn should_replace_recipients() {
        // Arrange
        let mut original_email = EmailMessage::new();
        original_email.to = vec![format!("{}@email.com", new_hyphenated_uuid()), "to_two@email.com".to_owned()];
        original_email.cc = vec![format!("{}@email.com", new_hyphenated_uuid()), "cc_two@email.com".to_owned()];
        original_email.bcc = vec![format!("{}@email.com", new_hyphenated_uuid())];

        let fixed_recipients = vec![format!("{}@email.com", new_hyphenated_uuid())];

        let email_service =
            FixedRecipientEmailClient::new(fixed_recipients.clone(), Arc::new(InMemoryEmailClient::new()));

        // Act
        email_service.send(original_email.clone()).await.unwrap();

        // Assert
        let emails = email_service.get_emails().unwrap();
        assert_eq!(1, emails.len());
        let received_email = &emails[0];

        assert_eq!(fixed_recipients, received_email.to);
        assert!(received_email.cc.is_empty());
        assert!(received_email.bcc.is_empty());
    }

    #[tokio::test]
    async fn should_add_original_recipients_info_to_the_email_text_if_present() {
        // Arrange
        let mut original_email = EmailMessage::new();
        original_email.to = vec![format!("{}@email.com", new_hyphenated_uuid()), "to_two@email.com".to_owned()];
        original_email.cc = vec![format!("{}@email.com", new_hyphenated_uuid()), "cc_two@email.com".to_owned()];
        original_email.bcc = vec![format!("{}@email.com", new_hyphenated_uuid())];

        let text = new_hyphenated_uuid();
        original_email.text = Some(text.clone());

        let fixed_recipients = vec![format!("{}@email.com", new_hyphenated_uuid())];

        let email_service =
            FixedRecipientEmailClient::new(fixed_recipients.clone(), Arc::new(InMemoryEmailClient::new()));

        // Act
        email_service.send(original_email.clone()).await.unwrap();

        // Assert
        let emails = email_service.get_emails().unwrap();
        assert_eq!(1, emails.len());
        let received_email = &emails[0];

        assert_eq!(
            Some(format!("{}\n{}", to_text(&original_email.to, &original_email.cc, &original_email.bcc), text)),
            received_email.text
        );
    }

    #[tokio::test]
    async fn should_add_original_recipients_info_to_the_email_html_if_present() {
        // Arrange
        let mut original_email = EmailMessage::new();
        original_email.to = vec![format!("{}@email.com", new_hyphenated_uuid()), "to_two@email.com".to_owned()];
        original_email.cc = vec![format!("{}@email.com", new_hyphenated_uuid()), "cc_two@email.com".to_owned()];
        original_email.bcc = vec![format!("{}@email.com", new_hyphenated_uuid())];

        let text = new_hyphenated_uuid();
        original_email.html = Some(text.clone());

        let fixed_recipients = vec![format!("{}@email.com", new_hyphenated_uuid())];

        let email_service =
            FixedRecipientEmailClient::new(fixed_recipients.clone(), Arc::new(InMemoryEmailClient::new()));

        // Act
        email_service.send(original_email.clone()).await.unwrap();

        // Assert
        let emails = email_service.get_emails().unwrap();
        assert_eq!(1, emails.len());
        let received_email = &emails[0];

        assert_eq!(
            Some(format!(
                "<pre>\n{}\n</pre>\n</br>\n{}",
                to_text(&original_email.to, &original_email.cc, &original_email.bcc),
                text
            )),
            received_email.html
        );
        assert_eq!(None, received_email.text);
    }

    #[tokio::test]
    async fn should_add_original_recipients_info_to_the_email_text_if_no_text_or_html_is_present() {
        // Arrange
        let mut original_email = EmailMessage::new();
        original_email.to = vec![format!("{}@email.com", new_hyphenated_uuid()), "to_two@email.com".to_owned()];
        original_email.cc = vec![format!("{}@email.com", new_hyphenated_uuid()), "cc_two@email.com".to_owned()];
        original_email.bcc = vec![format!("{}@email.com", new_hyphenated_uuid())];

        let fixed_recipients = vec![format!("{}@email.com", new_hyphenated_uuid())];

        let email_service =
            FixedRecipientEmailClient::new(fixed_recipients.clone(), Arc::new(InMemoryEmailClient::new()));

        // Act
        email_service.send(original_email.clone()).await.unwrap();

        // Assert
        let emails = email_service.get_emails().unwrap();
        assert_eq!(1, emails.len());
        let received_email = &emails[0];

        assert_eq!(Some(to_text(&original_email.to, &original_email.cc, &original_email.bcc)), received_email.text);
        assert_eq!(None, received_email.html);
    }

    #[tokio::test]
    async fn should_add_original_to_info_to_the_email_subject_if_present() {
        // Arrange
        let mut original_email = EmailMessage::new();
        original_email.to = vec![format!("{}@email.com", new_hyphenated_uuid()), "to_two@email.com".to_owned()];
        original_email.cc = vec![format!("{}@email.com", new_hyphenated_uuid()), "cc_two@email.com".to_owned()];
        original_email.bcc = vec![];

        let subject = new_hyphenated_uuid();
        original_email.subject = Some(subject.clone());

        let fixed_recipients = vec![format!("{}@email.com", new_hyphenated_uuid())];
        let email_service =
            FixedRecipientEmailClient::new(fixed_recipients.clone(), Arc::new(InMemoryEmailClient::new()));

        // Act
        email_service.send(original_email.clone()).await.unwrap();

        // Assert
        let emails = email_service.get_emails().unwrap();
        assert_eq!(1, emails.len());
        let received_email = &emails[0];

        assert_eq!(Some(to_subject(&subject, &original_email.to)), received_email.subject);
    }

    #[tokio::test]
    async fn should_add_original_to_info_to_the_email_subject_even_if_not_present() {
        // Arrange
        let mut original_email = EmailMessage::new();
        original_email.to = vec![format!("{}@email.com", new_hyphenated_uuid()), "to_two@email.com".to_owned()];
        original_email.cc = vec![format!("{}@email.com", new_hyphenated_uuid()), "cc_two@email.com".to_owned()];
        original_email.bcc = vec![];

        let fixed_recipients = vec![format!("{}@email.com", new_hyphenated_uuid())];
        let email_service =
            FixedRecipientEmailClient::new(fixed_recipients.clone(), Arc::new(InMemoryEmailClient::new()));

        // Act
        email_service.send(original_email.clone()).await.unwrap();

        // Assert
        let emails = email_service.get_emails().unwrap();
        assert_eq!(1, emails.len());
        let received_email = &emails[0];

        assert_eq!(Some(to_subject("", &original_email.to)), received_email.subject);
    }
}
