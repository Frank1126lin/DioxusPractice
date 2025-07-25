#![allow(warnings)]
#![cfg_attr(not(test), windows_subsystem = "windows")]

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder, LogicalSize};

mod components;
mod models;
mod service;

use components::{Inbox, Composer, Sidebar, EmailDetail};
use components::layout_resizer::{ResizeData, ResizeHandle, use_resize_state};
use models::email::{EmailAccount, AccountList};
use models::Email;
use components::login_page::LoginPage;

#[derive(Clone, PartialEq)]
pub enum Page {
    Inbox,
    Sent,
    Drafts,
    Deleted, // 新增已删除页面
    Spam,    // 新增垃圾邮件页面
    Compose,
}

// 将 CSS 作为静态字符串包含
static MAIN_CSS: &str = include_str!("..\\assets\\main.css");

fn main() {
    let config = Config::new()
        .with_disable_context_menu(false)  // 允许右键菜单
        .with_window(
            WindowBuilder::new()
                .with_title("RustMail")
                .with_resizable(true)
                .with_inner_size(LogicalSize::new(1200.0, 800.0))  // 初始窗口大小
                .with_maximized(true),  // 启动时最大化窗口
        );
    
    dioxus_desktop::launch::launch(
        App,
        Vec::new(),
        vec![Box::new(config)],
    );
}

#[component]
pub fn App() -> Element {
    let mut current_page = use_signal(|| Page::Inbox);
    let mut accounts = use_signal(|| AccountList::new());
    let mut current_account = use_signal(|| None::<EmailAccount>);
    let mut show_login = use_signal(|| true);
    let mut show_middle_column = use_signal(|| true);
    let mut selected_email = use_signal(|| None::<Email>);

    // 获取拖拽状态 - 添加 mut 关键字
    let (mut resize_data, mut column_widths) = use_resize_state();

    let on_login = {
        let mut accounts = accounts.clone();
        let mut current_account = current_account.clone();
        let mut show_login = show_login.clone();
        
        move |account: EmailAccount| {
            accounts.write().push(account.clone());
            current_account.set(Some(account));
            show_login.set(false);
        }
    };

    // 在读取 current_page 后更新中间栏可见性
    use_effect(move || {
        // 当页面为 Compose 时隐藏中间栏
        show_middle_column.set(*current_page.read() != Page::Compose);
    });

    // 处理鼠标移动事件 - 用于实时调整宽度
    let handle_mouse_move = move |e: MouseEvent| {
        let data = resize_data.read().clone();
        
        if data.is_dragging {
            if let Some(element_id) = &data.element_id {
                // 修正: 使用正确的鼠标位置获取方法
                let current_x = e.data.client_coordinates().x;
                let delta_x = current_x - data.start_x;
                
                // 计算百分比变化
                let delta_percent = delta_x / data.container_width * 100.0;
                
                // 获取当前宽度
                let (mut sidebar_width, mut email_list_width, mut content_width) = *column_widths.read();
                
                if element_id == "sidebar" {
                    // 调整侧边栏宽度
                    let new_width = (data.start_width / data.container_width * 100.0 + delta_percent)
                        .max(10.0)  // 最小宽度10%
                        .min(20.0); // 最大宽度20%
                    
                    // 计算变化量
                    let change = new_width - sidebar_width;
                    
                    // 更新宽度，保持总和为100%
                    sidebar_width = new_width;
                    email_list_width -= change;
                    
                    // 确保设置三元组
                    column_widths.set((sidebar_width, email_list_width, content_width));
                    
                } else if element_id == "email-list" {
                    // 调整邮件列表宽度
                    let new_width = (data.start_width / data.container_width * 100.0 + delta_percent)
                        .max(20.0)  // 最小宽度20%
                        .min(30.0); // 最大宽度60%
                    
                    // 计算变化量
                    let change = new_width - email_list_width;
                    
                    // 更新宽度，保持总和为100%
                    email_list_width = new_width;
                    content_width -= change;
                    
                    // 确保设置三元组
                    column_widths.set((sidebar_width, email_list_width, content_width));
                }
            }
        }
    };

    // 处理鼠标松开事件 - 结束拖拽
    let handle_mouse_up = move |_: MouseEvent| {
        let data = resize_data.read().clone();
        if data.is_dragging {
            // 重置拖拽状态
            resize_data.set(ResizeData::default());
        }
    };

    // 修改主界面布局以匹配QQ邮箱风格

    rsx! {
        // 使用 style 标签内联 CSS，而不是外部链接
        style { "{MAIN_CSS}" }
        
        // 全局事件监听
        div {
            class: if resize_data.read().is_dragging { "app-container resizing" } else { "app-container" },
            // 添加全局鼠标事件监听
            onmousemove: handle_mouse_move,
            onmouseup: handle_mouse_up,
            
            if *show_login.read() {
                div {
                    class: "login-container",
                    LoginPage { on_login: on_login }
                }
            } else {
                // 顶部导航栏 - 修改移除图片
                header {
                    class: "top-navbar",
                    div {
                        class: "logo",
                        // 移除 img 标签
                        h1 { "RustMail" }
                    }
                    
                    // 搜索框
                    div {
                        class: "search-bar",
                        input {
                            type: "text",
                            placeholder: "搜索",
                        }
                    }
                    
                    // 用户信息 - 移除头像和下拉箭头
                    div {
                        class: "user-info",
                        span { "{current_account.read().as_ref().map_or(String::new(), |acc| acc.address.clone())}" }
                    }
                }
                
                // 主体内容区
                div {
                    class: "mail-content",
                    
                    // 左侧工具栏
                    nav {
                        id: "sidebar",
                        class: "sidebar",
                        style: "width: {column_widths.read().0}%",
                        
                        // 写邮件按钮
                        button {
                            class: "compose-button",
                            onclick: move |_| current_page.set(Page::Compose),
                            span { class: "icon", "✉" }
                            "写信"
                        }
                        
                        // 文件夹列表
                        Sidebar {
                            current_page: current_page.read().clone(),
                            on_nav: move |page| current_page.set(page),
                            accounts: accounts.read().clone(),
                            current_account: current_account.read().clone(),
                            on_switch_account: move |acc| current_account.set(Some(acc)),
                        }
                        
                        // 添加调整手柄
                        ResizeHandle {
                            id: "sidebar".to_string(),
                            resize_data: resize_data.clone(),
                            column_widths: column_widths.clone()
                        }
                    }
                    
                    // 中间列 - 邮件列表
                    if *show_middle_column.read() {
                        div {
                            id: "email-list",
                            class: "email-list",
                            style: "width: {column_widths.read().1}%",
                            
                            // 邮件列表内容
                            match *current_page.read() {
                                Page::Inbox => rsx!(Inbox { 
                                    account: current_account.read().clone(),
                                    on_email_selected: move |email: Email| {
                                        selected_email.set(Some(email));
                                    }
                                }),
                                Page::Sent => rsx!(div { class: "empty-state", "已发送邮件（待实现）" }),
                                Page::Drafts => rsx!(div { class: "empty-state", "草稿箱（待实现）" }),
                                Page::Deleted => rsx!(div { class: "empty-state", "已删除邮件（待实现）" }),
                                Page::Spam => rsx!(div { class: "empty-state", "垃圾邮件（待实现）" }),
                                _ => rsx!(div { "" }),
                            }
                            
                            // 添加调整手柄
                            ResizeHandle {
                                id: "email-list".to_string(),
                                resize_data: resize_data.clone(),
                                column_widths: column_widths.clone()
                            }
                        }
                    }
                    
                    // 右侧内容区 - 恢复写邮件界面的原美化版本
                    div {
                        class: if *show_middle_column.read() {
                            "content-panel"
                        } else {
                            "content-panel expanded full-width"
                        },
                        style: {
                            // 当为写邮件界面时，设置样式以铺满整个区域
                            if *current_page.read() == Page::Compose {
                                "padding: 0; display: flex;"
                            } else {
                                ""
                            }
                        },
                        match *current_page.read() {
                            Page::Compose => rsx!(
                                Composer { 
                                    account: current_account.read().clone(),
                                    full_width: !*show_middle_column.read(),
                                }
                            ),
                            _ => {
                                // 显示选中的邮件详情或欢迎信息
                                if let Some(email) = selected_email.read().as_ref() {
                                    rsx!(EmailDetail { email: email.clone() })
                                } else {
                                    rsx!(
                                        div { 
                                            class: "welcome-message",
                                            h2 { "欢迎使用 RustMail" }
                                            p { "请从左侧选择邮件查看详情，或点击「写邮件」" }
                                        }
                                    )
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}