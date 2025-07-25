pub mod imap_client;
pub mod smtp_client;
pub mod local_storage;


pub use imap_client::fetch_emails;
pub use smtp_client::send_email;
pub use local_storage::{LocalStorage, LayoutSettings};