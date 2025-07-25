use dioxus::prelude::*;
use crate::models::email::EmailAccount;
use crate::service::imap_client;

#[derive(Props, PartialEq, Clone)]
pub struct InboxProps {
    pub account: Option<EmailAccount>,
}

pub fn Inbox(props: InboxProps) -> Element {
    let mut emails = use_signal(|| None);
    let mut selected_index = use_signal(|| 0);
    let mut is_loading = use_signal(|| true);

    // 加载邮件数据
    {
        let mut emails_setter = emails.clone();
        let mut loading_setter = is_loading.clone();
        let account = props.account.clone();
        
        use_effect(move || {
            loading_setter.set(true);
            
            // 在 spawn 之前克隆 account
            let account_clone = account.clone();
            
            spawn(async move {
                if let Some(acc) = account_clone {
                    let result = imap_client::fetch_emails(
                        &acc.imap_server,
                        acc.imap_port,
                        &acc.address,
                        &acc.password,
                        acc.use_tls,
                    ).await;
                    
                    match result {
                        Ok(fetched_emails) => {
                            println!("成功获取到 {} 封邮件", fetched_emails.len());
                            emails_setter.set(Some(fetched_emails));
                        },
                        Err(e) => {
                            println!("获取邮件失败: {}", e);
                            emails_setter.set(Some(Vec::new()));
                        }
                    }
                    loading_setter.set(false);
                } else {
                    println!("没有配置邮箱账户");
                    loading_setter.set(false);
                }
            });
            
            ()
        });
    }

    // 使用 use_callback 创建刷新函数，这样它可以被多次使用
    // 添加 move 关键字来获取所有变量的所有权
    let refresh_emails = use_callback(move |_| {
        is_loading.set(true);
        
        let account = props.account.clone();
        let mut emails_setter = emails.clone();
        let mut loading_setter = is_loading.clone();
        
        spawn(async move {
            if let Some(acc) = account {
                let result = imap_client::fetch_emails(
                    &acc.imap_server,
                    acc.imap_port,
                    &acc.address,
                    &acc.password,
                    acc.use_tls,
                ).await;
                
                match result {
                    Ok(fetched_emails) => {
                        println!("成功获取到 {} 封邮件", fetched_emails.len());
                        emails_setter.set(Some(fetched_emails));
                    },
                    Err(e) => {
                        println!("获取邮件失败: {}", e);
                        emails_setter.set(Some(Vec::new())); 
                    }
                }
                loading_setter.set(false);
            } else {
                println!("没有配置邮箱账户");
                loading_setter.set(false);
            }
        });
    });

    // 创建一个单独的 emails_copy 用于读取操作，避免与 refresh_emails 内部的闭包冲突
    let emails_copy = emails.clone();
    let is_loading_copy = is_loading.clone();

    // 主渲染函数
    rsx! {
        div {
            class: "inbox-container",
            
            // 添加刷新按钮
            div {
                class: "email-list-toolbar",
                
                // 刷新按钮
                button {
                    class: "action-btn",
                    onclick: move |evt| refresh_emails.call(evt),
                    "🔄 刷新"
                }
            }
            
            // 邮件列表
            div {
                class: "email-items",
                
                if *is_loading_copy.read() {
                    div {
                        class: "loading-indicator",
                        "加载中..."
                    }
                } else if let Some(email_list) = emails_copy.read().as_ref() {
                    if email_list.is_empty() {
                        div {
                            class: "empty-state",
                            div {
                                class: "empty-icon",
                                "📭"
                            }
                            p { "没有邮件" }
                            p { class: "empty-hint", "收到的邮件会显示在这里" }
                        }
                    } else {
                        {email_list.iter().enumerate().map(|(index, email)| {
                            let is_selected = *selected_index.read() == index;
                            
                            rsx! {
                                div {
                                    key: "{index}",
                                    class: if is_selected { "email-item selected" } else { "email-item" },
                                    onclick: move |_| selected_index.set(index),
                                    
                                    div {
                                        class: "email-checkbox-wrapper",
                                        input { r#type: "checkbox" }
                                    }
                                    div {
                                        class: "email-content",
                                        div {
                                            class: "email-row",
                                            div { class: "email-sender", "{email.from}" }
                                            div { class: "email-date", "{email.date}" }
                                        }
                                        div {
                                            class: "email-subject",
                                            "{email.subject}"
                                        }
                                    }
                                }
                            }
                        })}
                    }
                } else {
                    div {
                        class: "error-state",
                        div {
                            class: "error-icon",
                            "❌"
                        }
                        p { "获取邮件失败" }
                        p { class: "error-hint", "请检查你的网络连接和邮箱设置" }
                        
                        // 重试按钮
                        button {
                            class: "btn btn-secondary",
                            onclick: move |evt| refresh_emails.call(evt),
                            "重试"
                        }
                    }
                }
            }
        }
    }
}