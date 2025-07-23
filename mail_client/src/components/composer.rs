use dioxus::prelude::*;
use crate::models::email::{EmailAccount, Email, EmailStatus};
use crate::service::smtp_client; // 添加 SMTP 客户端模块导入

#[derive(Props, PartialEq, Clone)]
pub struct ComposerProps {
    pub account: Option<EmailAccount>,
    #[props(default = false)]
    pub full_width: bool,
}

// 恢复美化后的写邮件界面组件
pub fn Composer(props: ComposerProps) -> Element {
    let account = props.account.clone();
    
    let mut to = use_signal(|| String::new());
    let mut cc = use_signal(|| String::new());
    let mut bcc = use_signal(|| String::new());
    let mut subject = use_signal(|| String::new());
    let mut body = use_signal(|| String::new());
    let mut attachments = use_signal(|| Vec::new());
    let mut sending = use_signal(|| false);
    let mut send_status = use_signal(|| EmailStatus::Draft);

    // 处理发送
    let on_send = move |_| {
        if to.read().trim().is_empty() {
            send_status.set(EmailStatus::Failed("收件人不能为空".to_string()));
            return;
        }
        
        sending.set(true);
        send_status.set(EmailStatus::Sending);
        
        // 异步发送
        spawn({
            let mut to = to.clone();
            let mut cc = cc.clone();
            let mut bcc = bcc.clone();
            let mut subject = subject.clone();
            let mut body = body.clone();
            let mut attachments = attachments.clone();
            let mut send_status = send_status.clone();
            let mut sending = sending.clone();
            let account = account.clone();
            
            async move {
                // 构建邮件
                if let Some(account) = account.as_ref() {
                    let email = Email {
                        id: "".to_string(),
                        from: account.address.clone(),
                        to: to.read().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                        cc: cc.read().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                        bcc: bcc.read().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                        subject: subject.read().clone(),
                        body: body.read().clone(),
                        date: chrono::Local::now().to_rfc3339(),
                        attachments: attachments.read().clone(),
                        status: EmailStatus::Draft, // 初始状态为草稿
                    };
                    
                    // 使用SMTP客户端发送邮件 - 移除 .await
                    let result = smtp_client::send_email(
                        &account.smtp_server,
                        account.smtp_port,
                        &account.address,
                        &account.password,
                        &email,
                        account.use_tls,
                    );
                    
                    match result {
                        Ok(_) => {
                            // 发送成功
                            send_status.set(EmailStatus::Sent);
                            
                            // 清空表单
                            to.set(String::new());
                            cc.set(String::new());
                            bcc.set(String::new());
                            subject.set(String::new());
                            body.set(String::new());
                            attachments.set(Vec::new());
                        }
                        Err(e) => {
                            // 发送失败
                            send_status.set(EmailStatus::Failed(e.to_string()));
                        }
                    }
                    
                    sending.set(false);
                } else {
                    send_status.set(EmailStatus::Failed("未配置账户".to_string()));
                    sending.set(false);
                }
            }
        });
    };

    // 美化后的写邮件界面
    rsx! {
        div {
            class: if props.full_width { "composer-container full-width" } else { "composer-container" },

            // 标题栏
            div {
                class: "composer-header",
                h2 { "写邮件 - {props.account.as_ref().map_or(String::new(), |acc| acc.address.clone())}" }
                if props.full_width {
                    button {
                        class: "close-btn",
                        "×"
                    }
                }
            }
            
            // 邮件表单
            div {
                class: "composer-form",
                
                // 收件人
                div {
                    class: "form-group",
                    label { r#for: "to", "收件人:" }
                    input {
                        id: "to",
                        class: "form-control",
                        r#type: "text",
                        value: "{to}",
                        oninput: move |e| to.set(e.value().to_string()),
                        placeholder: "多个收件人请用逗号分隔"
                    }
                }
                
                // 抄送
                div {
                    class: "form-group",
                    label { r#for: "cc", "抄送:" }
                    input {
                        id: "cc",
                        class: "form-control",
                        r#type: "text",
                        value: "{cc}",
                        oninput: move |e| cc.set(e.value().to_string()),
                        placeholder: "多个抄送请用逗号分隔"
                    }
                }
                
                // 密送
                div {
                    class: "form-group",
                    label { r#for: "bcc", "密送:" }
                    input {
                        id: "bcc",
                        class: "form-control",
                        r#type: "text",
                        value: "{bcc}",
                        oninput: move |e| bcc.set(e.value().to_string()),
                        placeholder: "多个密送请用逗号分隔"
                    }
                }
                
                // 主题
                div {
                    class: "form-group",
                    label { r#for: "subject", "主题:" }
                    input {
                        id: "subject",
                        class: "form-control",
                        r#type: "text",
                        value: "{subject}",
                        oninput: move |e| subject.set(e.value().to_string()),
                        placeholder: "请输入主题"
                    }
                }
                
                // 正文
                div {
                    class: "form-group",
                    label { r#for: "body", "正文:" }
                    textarea {
                        id: "body",
                        class: "form-textarea",
                        value: "{body}",
                        oninput: move |e| body.set(e.value().to_string()),
                        rows: "15",
                        placeholder: "请输入邮件正文..."
                    }
                }
                
                // 底部工具栏
                div {
                    class: "composer-footer",
                    // 左侧：附件按钮
                    div {
                        class: "attachment-section",
                        button {
                            class: "btn btn-secondary",
                            r#type: "button",
                            "📎 添加附件"
                        }
                        // 附件列表显示
                        if !attachments.read().is_empty() {
                            div {
                                class: "attachment-list",
                                "附件列表将显示在这里"
                            }
                        }
                    }
                    
                    // 右侧：状态与发送按钮
                    div {
                        class: "send-section",
                        // 状态提示
                        span {
                            class: "status-message",
                            match &*send_status.read() {
                                EmailStatus::Draft => "",
                                EmailStatus::Sending => "发送中...",
                                EmailStatus::Sent => "已发送",
                                EmailStatus::Failed(e) => e,
                            }
                        }
                        
                        // 发送按钮
                        button {
                            class: "btn btn-primary",
                            r#type: "button",
                            onclick: on_send,
                            disabled: "{*sending.read()}",
                            if *sending.read() { "发送中..." } else { "发送" }
                        }
                    }
                }
            }
        }
    }
}