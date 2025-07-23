use crate::models::{Email, Attachment};
use lettre::{
    Message, SmtpTransport, Transport,
    transport::smtp::authentication::Credentials,
    message::{header, MultiPart, SinglePart, Attachment as LettreAttachment, Mailbox}
};

pub fn send_email(
    smtp_server: &str,
    smtp_port: u16,
    username: &str,
    password: &str,
    email: &Email,
    use_tls: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let creds = Credentials::new(username.to_string(), password.to_string());

    let mut builder = Message::builder().from(username.parse()?);

    for addr in &email.to {
        builder = builder.to(addr.parse::<Mailbox>()?);
    }
    for addr in &email.cc {
        builder = builder.cc(addr.parse::<Mailbox>()?);
    }
    for addr in &email.bcc {
        builder = builder.bcc(addr.parse::<Mailbox>()?);
    }

    builder = builder.subject(&email.subject);

    let mut multipart = MultiPart::mixed().singlepart(
        SinglePart::plain(email.body.clone())
    );

    for att in &email.attachments {
        multipart = multipart.singlepart(
            LettreAttachment::new(att.filename.clone())
                .body(att.data.clone(), att.content_type.parse()?),
        );
    }

    let email_builder = builder.multipart(multipart)?;

    let mailer = if use_tls {
        SmtpTransport::relay(smtp_server)?
            .port(smtp_port)
            .credentials(creds)
            .build()
    } else {
        SmtpTransport::builder_dangerous(smtp_server)
            .port(smtp_port)
            .credentials(creds)
            .build()
    };

    mailer.send(&email_builder)?;
    Ok(())
}