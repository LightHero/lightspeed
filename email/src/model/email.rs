use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EmailAttachment {
    pub path: String,
    pub filename: Option<String>,
    pub mime_type: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EmailMessage {
    pub from: Option<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: Option<String>,
    pub text: Option<String>,
    pub html: Option<String>,
    // pub inline_attachment: Vec<MailAttachment>,
    pub attachment: Vec<EmailAttachment>,
}

impl Default for EmailMessage {
    fn default() -> Self {
        EmailMessage {
            from: None,
            to: vec![],
            cc: vec![],
            bcc: vec![],
            subject: None,
            text: None,
            html: None,
            attachment: vec![],
        }
    }
}

impl EmailMessage {
    pub fn new() -> Self {
        EmailMessage::default()
    }
}
