use dioxus::prelude::*;
use dioxus::events::MouseEvent;  // 添加这一行
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use crate::models::email::EmailAccount;
use serde_json;

#[derive(Serialize, Deserialize, Default)]
struct LoginInfo {
    address: String,
    password: String,
    imap_server: String,
    imap_port: u16,
    smtp_server: String,
    smtp_port: u16,
    use_tls: bool,
}

#[derive(Clone, PartialEq)]
pub enum LoginStatus {
    Idle,
    Checking,
    Success,
    Failed(String),
}

#[derive(Props, PartialEq, Clone)]
pub struct LoginPageProps {
    pub on_login: EventHandler<EmailAccount>,
}

pub fn LoginPage(props: LoginPageProps) -> Element {
    let mut address = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut imap_server = use_signal(|| "imap.example.com".to_string());
    let mut imap_port = use_signal(|| 993u16);
    let mut smtp_server = use_signal(|| "smtp.example.com".to_string());
    let mut smtp_port = use_signal(|| 587u16);
    let mut use_tls = use_signal(|| true);
    let mut status = use_signal(|| LoginStatus::Idle);
    let mut show_form = use_signal(|| true);
    let mut auto_login_attempted = use_signal(|| false);

    // 尝试自动登录
    let auto_login = {
        let mut auto_login_attempted = auto_login_attempted.clone();
        let mut address = address.clone();
        let mut password = password.clone();
        let mut imap_server = imap_server.clone();
        let mut imap_port = imap_port.clone();
        let mut smtp_server = smtp_server.clone();
        let mut smtp_port = smtp_port.clone();
        let mut use_tls = use_tls.clone();
        let mut status = status.clone();
        let mut show_form = show_form.clone();
        let mut on_login = props.on_login.clone();
        
        move || {
            // 防止重复尝试自动登录
            if *auto_login_attempted.read() {
                return;
            }
            auto_login_attempted.set(true);
            
            // 尝试加载保存的账户信息
            if let Some(info) = load_login_info() {
                // 如果所有必要信息都已填写，则自动登录
                if !info.address.is_empty() && !info.password.is_empty() {
                    // 设置状态
                    status.set(LoginStatus::Checking);
                    
                    // 设置表单值（以防用户需要手动登录）
                    address.set(info.address.clone());
                    password.set(info.password.clone());
                    imap_server.set(info.imap_server.clone());
                    imap_port.set(info.imap_port);
                    smtp_server.set(info.smtp_server.clone());
                    smtp_port.set(info.smtp_port);
                    use_tls.set(info.use_tls);
                    
                    // 调用登录回调
                    on_login.call(EmailAccount {
                        address: info.address,
                        password: info.password,
                        imap_server: info.imap_server,
                        imap_port: info.imap_port,
                        smtp_server: info.smtp_server,
                        smtp_port: info.smtp_port,
                        use_tls: info.use_tls,
                    });
                    
                    status.set(LoginStatus::Success);
                    show_form.set(false); // 隐藏登录表单，因为我们已经自动登录
                } else {
                    // 如果信息不完整，则显示登录表单
                    show_form.set(true);
                }
            } else {
                // 没有保存的信息，显示登录表单
                show_form.set(true);
            }
        }
    };

    // 自动登录尝试
    use_hook(auto_login);

    let try_login = {
        let mut address = address.clone();
        let mut password = password.clone();
        let mut imap_server = imap_server.clone();
        let mut imap_port = imap_port.clone();
        let mut smtp_server = smtp_server.clone();
        let mut smtp_port = smtp_port.clone();
        let mut use_tls = use_tls.clone();
        let mut status = status.clone();
        let mut on_login = props.on_login.clone();
        move |_| {
            status.set(LoginStatus::Checking);
            let address = address.read().clone();
            let password = password.read().clone();
            let imap_server = imap_server.read().clone();
            let imap_port = *imap_port.read();
            let smtp_server = smtp_server.read().clone();
            let smtp_port = *smtp_port.read();
            let use_tls = *use_tls.read();

            // 这里可以加异步校验逻辑
            if address.is_empty() || password.is_empty() {
                status.set(LoginStatus::Failed("邮箱和密码不能为空".to_string()));
                return;
            }

            // 保存登录信息到本地
            let info = LoginInfo {
                address: address.clone(),
                password: password.clone(),
                imap_server: imap_server.clone(),
                imap_port,
                smtp_server: smtp_server.clone(),
                smtp_port,
                use_tls,
            };
            save_login_info(&info);

            // 这里直接调用 on_login
            on_login.call(EmailAccount {
                address,
                password,
                imap_server,
                imap_port,
                smtp_server,
                smtp_port,
                use_tls,
            });
            status.set(LoginStatus::Success);
        }
    };

    // 清除保存的登录信息并显示表单
    let show_login_form = {
        let mut show_form = show_form.clone();
        move |_: MouseEvent| {  // 添加 MouseEvent 类型
            show_form.set(true);
        }
    };

    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-100 to-purple-200",
            
            // 仅当 show_form 为 true 时显示登录表单
            if *show_form.read() {
                div {
                    class: "w-full max-w-md bg-white rounded-3xl shadow-2xl p-10 space-y-8",
                    h2 {
                        class: "text-4xl font-bold text-center text-blue-700 mb-4",
                        "邮箱账户登录"
                    }
                    hr { class: "mb-6 border-blue-200" }
                    div { class: "space-y-6",
                        div {
                            label { class: "block text-gray-700 font-semibold mb-2", "邮箱地址" }
                            input {
                                class: "w-full px-4 py-3 border border-gray-300 rounded-full focus:outline-none focus:ring-2 focus:ring-blue-400 text-lg",
                                value: "{address}",
                                oninput: move |e| address.set(e.value().to_string()),
                                placeholder: "your@email.com"
                            }
                        }
                        div {
                            label { class: "block text-gray-700 font-semibold mb-2", "密码" }
                            input {
                                r#type: "password",
                                class: "w-full px-4 py-3 border border-gray-300 rounded-full focus:outline-none focus:ring-2 focus:ring-blue-400 text-lg",
                                value: "{password}",
                                oninput: move |e| password.set(e.value().to_string()),
                                placeholder: "请输入密码"
                            }
                        }
                        div { class: "flex space-x-4",
                            div { class: "flex-1",
                                label { class: "block text-gray-700 font-semibold mb-2", "IMAP服务器" }
                                input {
                                    class: "w-full px-4 py-3 border border-gray-300 rounded-full focus:outline-none focus:ring-2 focus:ring-blue-400 text-lg",
                                    value: "{imap_server}",
                                    oninput: move |e| imap_server.set(e.value().to_string()),
                                    placeholder: "imap.example.com"
                                }
                            }
                            div { class: "w-28",
                                label { class: "block text-gray-700 font-semibold mb-2", "端口" }
                                input {
                                    r#type: "number",
                                    class: "w-full px-4 py-3 border border-gray-300 rounded-full focus:outline-none focus:ring-2 focus:ring-blue-400 text-lg",
                                    value: "{imap_port}",
                                    oninput: move |e| imap_port.set(e.value().parse().unwrap_or(993))
                                }
                            }
                        }
                        div { class: "flex space-x-4",
                            div { class: "flex-1",
                                label { class: "block text-gray-700 font-semibold mb-2", "SMTP服务器" }
                                input {
                                    class: "w-full px-4 py-3 border border-gray-300 rounded-full focus:outline-none focus:ring-2 focus:ring-blue-400 text-lg",
                                    value: "{smtp_server}",
                                    oninput: move |e| smtp_server.set(e.value().to_string()),
                                    placeholder: "smtp.example.com"
                                }
                            }
                            div { class: "w-28",
                                label { class: "block text-gray-700 font-semibold mb-2", "端口" }
                                input {
                                    r#type: "number",
                                    class: "w-full px-4 py-3 border border-gray-300 rounded-full focus:outline-none focus:ring-2 focus:ring-blue-400 text-lg",
                                    value: "{smtp_port}",
                                    oninput: move |e| smtp_port.set(e.value().parse().unwrap_or(587))
                                }
                            }
                        }
                        div { class: "flex items-center space-x-3 mt-2",
                            input {
                                r#type: "checkbox",
                                checked: *use_tls.read(),
                                oninput: move |e| use_tls.set(e.value() == "on"),
                                class: "rounded-full border-gray-300 focus:ring-blue-400"
                            }
                            label { class: "text-gray-700 text-lg", "使用TLS" }
                        }
                    }
                    div {
                        button {
                            class: "w-full py-3 mt-4 bg-blue-600 hover:bg-blue-700 text-white font-bold rounded-full shadow-lg text-lg transition",
                            onclick: try_login,
                            "登录"
                        }
                    }
                    div { class: "text-center mt-4 min-h-[1.5em]",
                        match &*status.read() {
                            LoginStatus::Idle => rsx!(span { "" }),
                            LoginStatus::Checking => rsx!(span { class: "text-blue-500", "正在校验..." }),
                            LoginStatus::Success => rsx!(span { class: "text-green-600", "登录成功！" }),
                            LoginStatus::Failed(e) => rsx!(span { class: "text-red-600", "{e}" }),
                        }
                    }
                }
            } else {
                div {
                    class: "text-center",
                    p { class: "text-lg text-blue-700", "正在登录，请稍候..." }
                }
            }
        }
    }
}

fn get_login_info_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push("login_info.json");
    path
}

fn load_login_info() -> Option<LoginInfo> {
    let path = get_login_info_path();
    if let Ok(data) = fs::read_to_string(path) {
        serde_json::from_str(&data).ok()
    } else {
        None
    }
}

fn save_login_info(info: &LoginInfo) {
    let path = get_login_info_path();
    if let Ok(data) = serde_json::to_string(info) {
        let _ = fs::write(path, data);
    }
}