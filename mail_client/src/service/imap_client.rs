use crate::models::{Email, Attachment};
use async_std::net::TcpStream;
use async_imap::{self, Client};
use async_native_tls::{TlsStream, TlsConnector};
use mailparse::{parse_mail, ParsedMail, MailHeaderMap}; 
use async_std::task;
use futures_util::stream::StreamExt;
use std::boxed::Box;
use chrono::{DateTime, Utc};

use crate::models::EmailStatus;

pub async fn fetch_emails(
    imap_server: &str,
    imap_port: u16,
    username: &str,
    password: &str,
    use_tls: bool,
) -> Result<Vec<Email>, Box<dyn std::error::Error + Send + Sync>> {
    println!("连接到 IMAP 服务器: {}:{}", imap_server, imap_port);
    
    // 使用枚举包装不同类型的会话
    enum ImapSession {
        Plain(async_imap::Session<TcpStream>),
        Tls(async_imap::Session<TlsStream<TcpStream>>),
    }
    
    // 创建会话
    let session = if use_tls {
        println!("使用 TLS 加密连接");
        let tcp_stream = TcpStream::connect((imap_server, imap_port)).await?;
        let tls = TlsConnector::new();
        let tls_stream = TlsConnector::connect(&tls, imap_server, tcp_stream).await?;
        let client = Client::new(tls_stream);
        
        println!("尝试登录...");
        let session = client
            .login(username, password)
            .await
            .map_err(|e| {
                println!("登录失败: {:?}", e);
                e.0
            })?;
        
        ImapSession::Tls(session)
    } else {
        println!("使用非加密连接");
        let tcp_stream = TcpStream::connect((imap_server, imap_port)).await?;
        let client = Client::new(tcp_stream);
        
        println!("尝试登录...");
        let session = client
            .login(username, password)
            .await
            .map_err(|e| {
                println!("登录失败: {:?}", e);
                e.0
            })?;
        
        ImapSession::Plain(session)
    };
    
    println!("登录成功，选择收件箱...");
    
    // 创建一个宏来处理不同类型的会话，避免代码重复
    macro_rules! handle_session {
        ($session:expr) => {{
            // 选择收件箱
            $session.select("INBOX").await?;
            
            let mut emails = Vec::new();
            
            println!("获取邮件列表...");
            let sequence = match $session.search("ALL").await {
                Ok(seq) => {
                    if seq.is_empty() {
                        println!("没有找到邮件");
                        return Ok(emails);
                    }
                    
                    let count = seq.len();
                    println!("找到 {} 封邮件", count);
                    
                    // 因为 seq 是 HashSet<u32>，我们需要将其转换为有序列表
                    let mut seq_vec: Vec<_> = seq.into_iter().collect();
                    seq_vec.sort_unstable(); // 排序，以便获取正确的序列号范围
                    
                    // 只获取最近的30封邮件
                    let start_idx = if count > 30 { count - 30 } else { 0 };
                    let range = if start_idx < count && start_idx < seq_vec.len() {
                        format!("{}:*", seq_vec[start_idx])
                    } else {
                        "1:*".to_string()
                    };
                    range
                },
                Err(e) => {
                    println!("搜索邮件失败: {}", e);
                    return Err(e.into());
                }
            };
            
            println!("获取邮件内容: {}", sequence);
            let mut fetches = $session.fetch(sequence, "RFC822").await?;
            
            while let Some(fetch) = fetches.next().await {
                match fetch {
                    Ok(fetch) => {
                        if let Some(body) = fetch.body() {
                            match parse_mail(body) {
                                Ok(parsed) => {
                                    // 修复：不要移动parsed.headers，而是使用引用
                                    let headers = &parsed.headers;
                                    
                                    let from = headers.get_first_header("From")
                                        .map(|h| h.get_value())
                                        .unwrap_or_else(|| String::from("未知发件人"));
                                        
                                    let subject = headers.get_first_header("Subject")
                                        .map(|h| h.get_value())
                                        .unwrap_or_else(|| String::from("无主题"));
                                        
                                    let date = headers.get_first_header("Date")
                                        .map(|h| h.get_value())
                                        .unwrap_or_else(|| String::from("未知日期"));
                                    
                                    // 现在 parsed 没有被部分移动，可以安全地借用
                                    let (body, attachments) = extract_body_and_attachments(&parsed);
                                    
                                    let to = headers.get_all_headers("To")
                                        .iter()
                                        .map(|h| h.get_value())
                                        .collect();
                                        
                                    let cc = headers.get_all_headers("Cc")
                                        .iter()
                                        .map(|h| h.get_value())
                                        .collect();
                                    
                                    let id = format!("{}", fetch.message);
                                    
                                    emails.push(Email {
                                        id,
                                        from,
                                        to,
                                        cc,
                                        bcc: vec![],
                                        subject,
                                        body,
                                        date,
                                        attachments,
                                        status: crate::models::EmailStatus::Draft,
                                    });
                                },
                                Err(e) => println!("解析邮件失败: {}", e),
                            }
                        }
                    },
                    Err(e) => println!("获取邮件错误: {}", e),
                }
            }
            
            // 修复: 确保在使用fetches后再调用logout，防止多次可变借用
            // 明确丢弃fetches，确保其生命周期结束
            drop(fetches);
            
            // 现在可以安全地调用logout
            let _ = $session.logout().await;
            
            // 按日期排序，最新的邮件在前面
            println!("成功获取 {} 封邮件", emails.len());
            emails.sort_by(|a, b| b.date.cmp(&a.date));
            
            Ok(emails)
        }}
    }
    
    // 修复: 将session声明为mut，以便在宏中可变借用
    match session {
        ImapSession::Plain(mut session) => handle_session!(session),
        ImapSession::Tls(mut session) => handle_session!(session),
    }
}

// extract_body_and_attachments 函数保持不变
fn extract_body_and_attachments(parsed: &ParsedMail) -> (String, Vec<Attachment>) {
    let mut body = String::new();
    let mut attachments = Vec::new();
    
    if parsed.ctype.mimetype.starts_with("text/") {
        if let Ok(text) = parsed.get_body() {
            body = text;
        }
    } else if parsed.ctype.mimetype == "multipart/alternative" || parsed.ctype.mimetype == "multipart/mixed" {
        for subpart in &parsed.subparts {
            if subpart.ctype.mimetype == "text/plain" {
                if let Ok(text) = subpart.get_body() {
                    body = text;
                }
            } else if subpart.ctype.mimetype == "text/html" {
                if let Ok(text) = subpart.get_body() {
                    body = text;
                }
            } else if !subpart.ctype.mimetype.starts_with("multipart/") {
                let filename = subpart.ctype.params.get("name")
                    .or_else(|| subpart.ctype.params.get("filename"))
                    .unwrap_or(&"未命名附件".to_string()).clone();
                
                if let Ok(content) = subpart.get_body_raw() {
                    attachments.push(Attachment {
                        filename,
                        content_type: subpart.ctype.mimetype.clone(),
                        data: content,
                    });
                }
            } else if subpart.ctype.mimetype.starts_with("multipart/") {
                let (inner_body, inner_attachments) = extract_body_and_attachments(subpart);
                if body.is_empty() {
                    body = inner_body;
                }
                attachments.extend(inner_attachments);
            }
        }
    }
    
    (body, attachments)
}

// 添加到 imap_client.rs 中的适当位置

// 添加一个新函数，支持从特定日期之后获取邮件
pub async fn fetch_new_emails(
    imap_server: &str,
    imap_port: u16,
    username: &str,
    password: &str,
    use_tls: bool,
    since_date: Option<DateTime<Utc>>, // 添加一个参数，只获取该日期之后的邮件
    exclude_ids: Vec<String>,         // 排除已有的邮件ID
) -> Result<Vec<Email>, Box<dyn std::error::Error + Send + Sync>> {
    println!("连接到 IMAP 服务器: {}:{}", imap_server, imap_port);
    
    // 使用枚举包装不同类型的会话
    enum ImapSession {
        Plain(async_imap::Session<TcpStream>),
        Tls(async_imap::Session<TlsStream<TcpStream>>),
    }
    
    // 创建会话 - 复用 fetch_emails 中的连接代码
    let session = if use_tls {
        println!("使用 TLS 加密连接");
        let tcp_stream = TcpStream::connect((imap_server, imap_port)).await?;
        let tls = TlsConnector::new();
        let tls_stream = TlsConnector::connect(&tls, imap_server, tcp_stream).await?;
        let client = Client::new(tls_stream);
        
        println!("尝试登录...");
        let session = client
            .login(username, password)
            .await
            .map_err(|e| {
                println!("登录失败: {:?}", e);
                e.0
            })?;
        
        ImapSession::Tls(session)
    } else {
        println!("使用非加密连接");
        let tcp_stream = TcpStream::connect((imap_server, imap_port)).await?;
        let client = Client::new(tcp_stream);
        
        println!("尝试登录...");
        let session = client
            .login(username, password)
            .await
            .map_err(|e| {
                println!("登录失败: {:?}", e);
                e.0
            })?;
        
        ImapSession::Plain(session)
    };
    
    println!("登录成功，选择收件箱...");
    
    // 创建一个宏来处理不同类型的会话，避免代码重复
    macro_rules! handle_session {
        ($session:expr) => {{
            // 选择收件箱
            $session.select("INBOX").await?;
            
            let mut emails = Vec::new();
            
            println!("获取邮件列表...");
            
            // 构建搜索命令
            let search_cmd = if let Some(date) = since_date {
                // 格式化日期为 IMAP 搜索格式 (DD-MMM-YYYY)
                let date_str = date.format("%d-%b-%Y").to_string();
                format!("SINCE {}", date_str)
            } else {
                "ALL".to_string()
            };
            
            println!("搜索邮件: {}", search_cmd);
            let sequence = match $session.search(&search_cmd).await {
                Ok(seq) => {
                    if seq.is_empty() {
                        println!("没有找到邮件");
                        return Ok(emails);
                    }
                    
                    let count = seq.len();
                    println!("找到 {} 封邮件", count);
                    
                    // 因为 seq 是 HashSet<u32>，我们需要将其转换为有序列表
                    let mut seq_vec: Vec<_> = seq.into_iter().collect();
                    seq_vec.sort_unstable(); // 排序，以便获取正确的序列号范围
                    
                    // 只获取最近的30封邮件
                    let start_idx = if count > 30 { count - 30 } else { 0 };
                    let range = if start_idx < count && start_idx < seq_vec.len() {
                        format!("{}:*", seq_vec[start_idx])
                    } else {
                        "1:*".to_string()
                    };
                    range
                },
                Err(e) => {
                    println!("搜索邮件失败: {}", e);
                    return Err(e.into());
                }
            };
            
            println!("获取邮件内容: {}", sequence);
            let mut fetches = $session.fetch(sequence, "RFC822").await?;
            
            while let Some(fetch) = fetches.next().await {
                match fetch {
                    Ok(fetch) => {
                        if let Some(body) = fetch.body() {
                            match parse_mail(body) {
                                Ok(parsed) => {
                                    // 使用引用而不是移动
                                    let headers = &parsed.headers;
                                    
                                    let from = headers.get_first_header("From")
                                        .map(|h| h.get_value())
                                        .unwrap_or_else(|| String::from("未知发件人"));
                                        
                                    let subject = headers.get_first_header("Subject")
                                        .map(|h| h.get_value())
                                        .unwrap_or_else(|| String::from("无主题"));
                                        
                                    let date = headers.get_first_header("Date")
                                        .map(|h| h.get_value())
                                        .unwrap_or_else(|| String::from("未知日期"));
                                    
                                    let (body, attachments) = extract_body_and_attachments(&parsed);
                                    
                                    let to = headers.get_all_headers("To")
                                        .iter()
                                        .map(|h| h.get_value())
                                        .collect();
                                        
                                    let cc = headers.get_all_headers("Cc")
                                        .iter()
                                        .map(|h| h.get_value())
                                        .collect();
                                    
                                    let id = format!("{}", fetch.message);
                                    
                                    // 检查是否在排除列表中
                                    if !exclude_ids.contains(&id) {
                                        emails.push(Email {
                                            id,
                                            from,
                                            to,
                                            cc,
                                            bcc: vec![],
                                            subject,
                                            body,
                                            date,
                                            attachments,
                                            status: crate::models::EmailStatus::Draft,
                                        });
                                    }
                                },
                                Err(e) => println!("解析邮件失败: {}", e),
                            }
                        }
                    },
                    Err(e) => println!("获取邮件错误: {}", e),
                }
            }
            
            // 确保在使用fetches后再调用logout，防止多次可变借用
            drop(fetches);
            
            // 现在可以安全地调用logout
            let _ = $session.logout().await;
            
            // 按日期排序，最新的邮件在前面
            println!("成功获取 {} 封新邮件", emails.len());
            emails.sort_by(|a, b| b.date.cmp(&a.date));
            
            Ok(emails)
        }}
    }
    
    // 根据会话类型调用相应的处理逻辑
    match session {
        ImapSession::Plain(mut session) => handle_session!(session),
        ImapSession::Tls(mut session) => handle_session!(session),
    }
}