use std::time::Duration;
use dioxus::prelude::*;
use crate::models::email::EmailAccount;
use crate::service::{imap_client, local_storage::LocalStorage};
use crate::models::Email;

#[derive(Props, PartialEq, Clone)]
pub struct InboxProps {
    pub account: Option<EmailAccount>,
}

pub fn Inbox(props: InboxProps) -> Element {
    let mut emails = use_signal(|| None);
    let mut selected_index = use_signal(|| 0);
    let mut is_loading = use_signal(|| true);
    let mut local_storage = use_signal(|| None::<LocalStorage>);
    let mut last_sync_time = use_signal(|| None::<String>);

    // 初始化本地存储
    {
        let mut local_storage_setter = local_storage.clone();
        
        use_effect(move || {
            match LocalStorage::new() {
                Ok(storage) => {
                    local_storage_setter.set(Some(storage));
                }
                Err(e) => {
                    println!("初始化本地存储失败: {}", e);
                }
            }
            
            ()
        });
    }

    // 加载本地邮件
    {
        let mut emails_setter = emails.clone();
        let mut loading_setter = is_loading.clone();
        let account = props.account.clone();
        let local_storage_reader = local_storage.clone();
        let mut last_sync_setter = last_sync_time.clone();
        
        use_effect(move || {
            loading_setter.set(true);
            
            if let (Some(acc), Some(storage)) = (account.clone(), local_storage_reader.read().as_ref()) {
                // 更新最后同步时间
                if let Some(last_sync) = storage.get_last_sync() {
                    last_sync_setter.set(Some(last_sync.format("%Y-%m-%d %H:%M:%S").to_string()));
                }
                
                // 尝试从本地加载
                match storage.load_emails(&acc, "INBOX") {
                    Ok(local_emails) => {
                        if !local_emails.is_empty() {
                            println!("从本地加载了 {} 封邮件", local_emails.len());
                            emails_setter.set(Some(local_emails));
                            loading_setter.set(false);
                            
                            // 然后在后台刷新
                            sync_with_server(acc.clone(), storage, emails_setter.clone(), last_sync_setter.clone());
                        } else {
                            // 本地没有邮件，直接从服务器获取
                            sync_with_server(acc.clone(), storage, emails_setter.clone(), last_sync_setter.clone());
                        }
                    }
                    Err(e) => {
                        println!("从本地加载邮件失败: {}", e);
                        // 从服务器获取
                        sync_with_server(acc.clone(), storage, emails_setter.clone(), last_sync_setter.clone());
                    }
                }
            } else {
                loading_setter.set(false);
                println!("没有配置邮箱账户或本地存储");
            }
            
            ()
        });
    }

    // 使用 use_future 进行定期同步
    {
        let account = props.account.clone();
        let local_storage_reader = local_storage.clone();
        let emails_setter = emails.clone();
        let mut last_sync_setter = last_sync_time.clone();
        
        // 使用 use_future 创建一个异步任务
        use_future(move || {
            let account_clone = account.clone();
            let local_storage_reader_clone = local_storage_reader.clone();
            let emails_setter_clone = emails_setter.clone();
            let last_sync_setter_clone = last_sync_setter.clone();
            
            async move {
                loop {
                    // 等待10分钟
                    async_std::task::sleep(Duration::from_secs(600)).await;
                    
                    if let (Some(acc), Some(storage)) = (account_clone.clone(), local_storage_reader_clone.read().as_ref()) {
                        println!("执行定时同步...");
                        sync_with_server(acc.clone(), storage, emails_setter_clone.clone(), last_sync_setter_clone.clone()).await;
                    }
                }
            }
        });
    }

    // 使用 use_callback 创建刷新函数
    let refresh_emails = use_callback(move |_| {
        is_loading.set(true);
        
        let account = props.account.clone();
        // 正确获取本地存储
        let mut local_storage_reader = local_storage.clone();
        let mut emails_setter = emails.clone();
        let mut loading_setter = is_loading.clone();
        let mut last_sync_setter = last_sync_time.clone();
        
        if let Some(acc) = account {
            // 从本地存储中获取实例
            if let Some(storage) = local_storage_reader.read().as_ref() {
                let storage_clone = storage.clone();
                // 直接从服务器同步最新邮件
                spawn(async move {
                    sync_with_server(acc, &storage_clone, emails_setter.clone(), last_sync_setter.clone()).await;
                    loading_setter.set(false);
                });
            } else {
                loading_setter.set(false);
            }
        } else {
            loading_setter.set(false);
        }
    });

    // 主渲染函数
    rsx! {
        div {
            class: "inbox-container",
            
            // 修改后的邮件列表工具栏
            div {
                class: "email-list-toolbar",
                
                // 显示最后同步时间 (居左)
                div {
                    class: "sync-info",
                    if let Some(time) = last_sync_time.read().as_ref() {
                        span {
                            class: "sync-time",
                            "上次同步: {time}"
                        }
                    } else {
                        span {
                            class: "sync-time",
                            "未同步"
                        }
                    }
                }
                
                // 刷新按钮 (居右)
                button {
                    class: "action-btn refresh-btn",
                    onclick: move |evt| refresh_emails.call(evt),
                    "🔄 刷新"
                }
            }
            
            // 邮件列表 (不变)
            div {
                class: "email-items",
                
                if *is_loading.read() {
                    div {
                        class: "loading-indicator",
                        "加载中..."
                    }
                } else if let Some(email_list) = emails.read().as_ref() {
                    if email_list.is_empty() {
                        // 空状态显示 (不变)
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
                        // 邮件列表项 (不变)
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
                    // 错误状态显示 (不变)
                    div {
                        class: "error-state",
                        div {
                            class: "error-icon",
                            "❌"
                        }
                        p { "获取邮件失败" }
                        p { class: "error-hint", "请检查你的网络连接和邮箱设置" }
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

// 帮助函数：与服务器同步
async fn sync_with_server(
    account: EmailAccount,
    storage: &LocalStorage,
    mut emails_setter: Signal<Option<Vec<Email>>>,
    mut last_sync_setter: Signal<Option<String>>,
) {
    println!("从服务器同步邮件...");
    
    // 获取已同步的邮件ID
    let synced_ids = storage.get_synced_ids(&account);
    // 获取最后同步时间
    let since_date = storage.get_last_sync();
    
    // 只获取新邮件
    match imap_client::fetch_new_emails(
        &account.imap_server,
        account.imap_port,
        &account.address,
        &account.password,
        account.use_tls,
        since_date,
        synced_ids,
    ).await {
        Ok(new_emails) => {
            println!("从服务器获取到 {} 封新邮件", new_emails.len());
            
            // 保存到本地
            if !new_emails.is_empty() {
                // 创建一个可变的本地存储副本
                // 使用 try_clone 方法而不是 clone
                let mut storage_mut = storage.clone();
                
                if let Err(e) = storage_mut.save_emails(&account, "INBOX", &new_emails) {
                    println!("保存邮件到本地失败: {}", e);
                }
                
                // 更新最后同步时间显示
                if let Some(last_sync) = storage_mut.get_last_sync() {
                    last_sync_setter.set(Some(last_sync.format("%Y-%m-%d %H:%M:%S").to_string()));
                }
            }
            
            // 合并本地邮件
            match storage.load_emails(&account, "INBOX") {
                Ok(all_emails) => {
                    emails_setter.set(Some(all_emails));
                },
                Err(e) => {
                    println!("加载合并后的邮件失败: {}", e);
                    // 至少显示新获取的邮件
                    if !new_emails.is_empty() {
                        emails_setter.set(Some(new_emails));
                    }
                }
            }
        },
        Err(e) => {
            println!("从服务器获取邮件失败: {}", e);
            // 仍然尝试加载本地邮件
            if let Ok(local_emails) = storage.load_emails(&account, "INBOX") {
                emails_setter.set(Some(local_emails));
            }
        }
    }
}