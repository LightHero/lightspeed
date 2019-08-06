use c3p0::Model;
use serde::{Deserialize, Serialize};

pub type EmailModel = Model<EmailData>;

#[derive(Clone, Serialize, Deserialize)]
pub struct EmailData {
    pub message: EmailMessage,
    pub created_at_epoch: i64,
}

#[derive(Clone, Serialize, Deserialize)]
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
        EmailMessage{
            from: None,
            to: vec![],
            cc: vec![],
            bcc: vec![],
            subject: None,
            text: None,
            html: None
        }
    }
}

impl EmailMessage {

    pub fn new() -> Self {
        EmailMessage::default()
    }

}