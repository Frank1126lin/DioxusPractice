use dioxus::prelude::*;
use dioxus_desktop::use_window;
use crate::Page;
use crate::models::email::EmailAccount;
use crate::components::Composer;

#[derive(Props, PartialEq, Clone)]
pub struct SidebarProps {
    pub current_page: Page,
    pub on_nav: EventHandler<Page>,
    pub accounts: Vec<EmailAccount>,
    pub current_account: Option<EmailAccount>,
    pub on_switch_account: EventHandler<EmailAccount>,
}

pub fn Sidebar(props: SidebarProps) -> Element {
    rsx! {
        div {
            class: "sidebar-content",
            
            // 文件夹菜单
            ul {
                class: "sidebar-menu",
                li {
                    class: "sidebar-menu-item",
                    div {
                        class: match props.current_page {
                            Page::Inbox => "sidebar-menu-link active",
                            _ => "sidebar-menu-link"
                        },
                        onclick: move |_| props.on_nav.call(Page::Inbox),
                        span { class: "icon", "📥" }
                        span { "收件箱" }
                    }
                }
                li {
                    class: "sidebar-menu-item",
                    div {
                        class: match props.current_page {
                            Page::Sent => "sidebar-menu-link active",
                            _ => "sidebar-menu-link"
                        },
                        onclick: move |_| props.on_nav.call(Page::Sent),
                        span { class: "icon", "📤" }
                        span { "已发送" }
                    }
                }
                li {
                    class: "sidebar-menu-item",
                    div {
                        class: match props.current_page {
                            Page::Drafts => "sidebar-menu-link active",
                            _ => "sidebar-menu-link"
                        },
                        onclick: move |_| props.on_nav.call(Page::Drafts),
                        span { class: "icon", "📝" }
                        span { "草稿箱" }
                    }
                }
                li {
                    class: "sidebar-menu-item",
                    div {
                        class: match props.current_page {
                            Page::Deleted => "sidebar-menu-link active",
                            _ => "sidebar-menu-link"
                        },
                        onclick: move |_| props.on_nav.call(Page::Deleted),
                        span { class: "icon", "🗑️" }
                        span { "已删除" }
                    }
                }
                li {
                    class: "sidebar-menu-item",
                    div {
                        class: match props.current_page {
                            Page::Spam => "sidebar-menu-link active",
                            _ => "sidebar-menu-link"
                        },
                        onclick: move |_| props.on_nav.call(Page::Spam),
                        span { class: "icon", "⚠️" }
                        span { "垃圾邮件" }
                    }
                }
            }
            
            // 移除底部账户信息区域
        }
    }
}



