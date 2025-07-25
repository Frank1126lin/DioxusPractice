use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmailAccount {
    pub address: String,
    pub password: String,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub imap_server: String,
    pub imap_port: u16,
    pub use_tls: bool,
}

pub type AccountList = Vec<EmailAccount>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Email {
    pub id: String,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub date: String,
    pub body: String,
    pub attachments: Vec<Attachment>,
    pub status: EmailStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EmailStatus {
    Draft,
    Sending,
    Sent,
    Failed(String),
}

use std::default::Default;

impl Default for EmailAccount {
    fn default() -> Self {
        EmailAccount {
            address: "".to_string(),
            password: "".to_string(),
            imap_server: "".to_string(),
            imap_port: 993,
            smtp_server: "".to_string(),
            smtp_port: 587,
            use_tls: true,
        }
    }
}