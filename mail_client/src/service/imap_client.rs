use crate::models::{Email, Attachment};
use async_imap::Session;
use async_native_tls::TlsConnector;
use mailparse::{parse_mail, ParsedMail};
use async_std::net::TcpStream;
use async_imap::types::UnsolicitedResponse;
use async_std::task;
use futures_util::stream::StreamExt;

pub async fn fetch_emails(
    imap_server: &str,
    imap_port: u16,
    username: &str,
    password: &str,
    use_tls: bool,
) -> Result<Vec<Email>, Box<dyn std::error::Error>> {
    let tcp_stream = TcpStream::connect((imap_server, imap_port)).await?;
    let tls = TlsConnector::new();
    let tls_stream = async_native_tls::TlsConnector::connect(&tls, imap_server, tcp_stream).await?;

    // 直接用 tls_stream，不用 compat
    let mut client = async_imap::Client::new(tls_stream);
    let mut session = client
        .login(username, password)
        .await
        .map_err(|e| e.0)?;

    session.select("INBOX").await?;

    let mut emails = Vec::new();

    {
        let mut fetches = session.fetch("1:*", "RFC822").await?;
        while let Some(fetch) = fetches.next().await {
            let fetch = fetch?;
            if let Some(body) = fetch.body() {
                let parsed = parse_mail(body)?;
                let (plain_body, attachments) = extract_body_and_attachments(&parsed);

                if let Some(envelope) = fetch.envelope() {
                    let subject = envelope.subject
                        .as_ref()
                        .map(|s| String::from_utf8_lossy(s).to_string())
                        .unwrap_or_default();
                    let from = envelope.from
                        .as_ref()
                        .and_then(|addrs| addrs.get(0))
                        .and_then(|addr| addr.mailbox.as_ref())
                        .map(|s| String::from_utf8_lossy(s).to_string())
                        .unwrap_or_default();
                    let date = envelope.date
                        .as_ref()
                        .map(|s| String::from_utf8_lossy(s).to_string())
                        .unwrap_or_default();

                    let to = envelope.to
                        .as_ref()
                        .map(|addrs| addrs.iter()
                            .filter_map(|addr| addr.mailbox.as_ref().map(|s| String::from_utf8_lossy(s).to_string()))
                            .collect())
                        .unwrap_or_else(Vec::new);
                    let cc = envelope.cc
                        .as_ref()
                        .map(|addrs| addrs.iter()
                            .filter_map(|addr| addr.mailbox.as_ref().map(|s| String::from_utf8_lossy(s).to_string()))
                            .collect())
                        .unwrap_or_else(Vec::new);
                    let bcc = envelope.bcc
                        .as_ref()
                        .map(|addrs| addrs.iter()
                            .filter_map(|addr| addr.mailbox.as_ref().map(|s| String::from_utf8_lossy(s).to_string()))
                            .collect())
                        .unwrap_or_else(Vec::new);

                    emails.push(Email {
                        id: fetch.message.to_string(),
                        from,
                        to,
                        cc,
                        bcc,
                        subject,
                        date,
                        body: plain_body,
                        attachments,
                        status: crate::models::email::EmailStatus::Draft,
                    });
                }
                // else: envelope 为 None，跳过该邮件
            }
        }
    } // fetches 在这里被 drop

    session.logout().await?;
    Ok(emails)
}

fn extract_body_and_attachments(parsed: &ParsedMail) -> (String, Vec<Attachment>) {
    let mut body = String::new();
    let mut attachments = Vec::new();

    if parsed.subparts.is_empty() {
        if let Ok(text) = parsed.get_body() {
            body = text;
        }
    } else {
        for part in &parsed.subparts {
            let cdisp = part.get_content_disposition();
            if cdisp.disposition == mailparse::DispositionType::Attachment {
                if let Some(filename) = cdisp.params.get("filename") {
                    let content_type = part.ctype.mimetype.clone();
                    let data = part.get_body_raw().unwrap_or_default();
                    attachments.push(Attachment {
                        filename: filename.clone(),
                        content_type,
                        data,
                    });
                }
            } else if part.ctype.mimetype == "text/plain" {
                if let Ok(text) = part.get_body() {
                    body = text;
                }
            }
        }
    }
    (body, attachments)
}