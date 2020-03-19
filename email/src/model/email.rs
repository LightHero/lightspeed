use serde::{Deserialize, Serialize};

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
    // pub attachment: Vec<MailAttachment>;
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
        }
    }
}

impl EmailMessage {
    pub fn new() -> Self {
        EmailMessage::default()
    }
}
