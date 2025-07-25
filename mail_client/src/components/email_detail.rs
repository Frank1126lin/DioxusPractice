use dioxus::prelude::*;
use crate::models::Email;

#[derive(Props, PartialEq, Clone)]
pub struct EmailDetailProps {
    pub email: Email,
}

pub fn EmailDetail(props: EmailDetailProps) -> Element {
    let email = &props.email;
    
    rsx! {
        div {
            class: "email-detail",
            
            // 邮件头部信息
            div {
                class: "email-header",
                h2 {
                    class: "email-subject",
                    "{email.subject}"
                }
                
                div {
                    class: "email-meta",
                    div {
                        class: "email-from",
                        strong { "发件人: " }
                        span { "{email.from}" }
                    }
                    
                    div {
                        class: "email-to",
                        strong { "收件人: " }
                        span { "{email.to.join(\", \")}" }
                    }
                    
                    if !email.cc.is_empty() {
                        div {
                            class: "email-cc",
                            strong { "抄送: " }
                            span { "{email.cc.join(\", \")}" }
                        }
                    }
                    
                    div {
                        class: "email-date",
                        strong { "日期: " }
                        span { "{email.date}" }
                    }
                }
                
                // 操作按钮
                div {
                    class: "email-actions",
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            // TODO: 实现回复功能
                        },
                        "回复"
                    }
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            // TODO: 实现转发功能
                        },
                        "转发"
                    }
                    button {
                        class: "btn btn-danger",
                        onclick: move |_| {
                            // TODO: 实现删除功能
                        },
                        "删除"
                    }
                }
            }
            
            // 邮件正文
            div {
                class: "email-body",
                div {
                    class: "email-content",
                    dangerous_inner_html: "{email.body}"
                }
            }
            
            // 附件列表
            if !email.attachments.is_empty() {
                div {
                    class: "email-attachments",
                    h3 { "附件:" }
                    div {
                        class: "attachment-list",
                        for attachment in &email.attachments {
                            div {
                                class: "attachment-item",
                                span {
                                    class: "attachment-icon",
                                    "📎"
                                }
                                span {
                                    class: "attachment-name",
                                    "{attachment.filename}"
                                }
                                span {
                                    class: "attachment-size",
                                    "({attachment.data.len()} bytes)"
                                }
                                button {
                                    class: "btn btn-link",
                                    onclick: move |_| {
                                        // TODO: 实现下载功能
                                    },
                                    "下载"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
