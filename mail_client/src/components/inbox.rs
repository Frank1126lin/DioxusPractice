use dioxus::prelude::*;
use crate::models::email::EmailAccount;
use crate::service::imap_client;

#[derive(Props, PartialEq, Clone)]
pub struct InboxProps {
    pub account: Option<EmailAccount>,
}

pub fn Inbox(props: InboxProps) -> Element {
    let emails = use_signal(|| None);
    let mut selected_index = use_signal(|| 0); // 跟踪选中的邮件索引
    let mut is_loading = use_signal(|| true); // 跟踪加载状态

    // 加载邮件数据
    if emails.read().is_none() {
        let mut emails_setter = emails.clone();
        let mut loading_setter = is_loading.clone();
        let account = props.account.clone();
        
        spawn(async move {
            if let Some(acc) = account {
                let result = imap_client::fetch_emails(
                    &acc.imap_server,
                    acc.imap_port,
                    &acc.address,
                    &acc.password,
                    acc.use_tls,
                ).await;
                
                emails_setter.set(Some(result.ok()));
                loading_setter.set(false); // 无论成功失败，加载完成
            } else {
                loading_setter.set(false); // 无账户信息，加载完成
            }
        });
    }

    // 主渲染函数
    rsx! {
        div {
            class: "inbox-container",
            
            // 邮件列表
            div {
                class: "email-items",
                
                // 只渲染实际的邮件数据
                if let Some(Some(email_list)) = emails.read().as_ref() {
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
                } else if !*is_loading.read() {
                    // 如果不在加载中但没有邮件，显示空状态
                    div {
                        class: "empty-state",
                        div {
                            class: "empty-icon",
                            "📭"
                        }
                        p { "没有邮件" }
                        p { class: "empty-hint", "收到的邮件会显示在这里" }
                    }
                }
            }
            
            // 显示加载状态
            if *is_loading.read() {
                div {
                    class: "loading-indicator",
                    "加载中..."
                }
            }
        }
    }
}