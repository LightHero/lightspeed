use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum EmailAttachment {
    Binary { body: Vec<u8>, filename: String, mime_type: String },
    FromFile { path: String, filename: Option<String>, mime_type: String },
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
    pub attachments: Vec<EmailAttachment>,
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
            attachments: vec![],
        }
    }
}

impl EmailMessage {
    pub fn new() -> Self {
        EmailMessage::default()
    }
}
