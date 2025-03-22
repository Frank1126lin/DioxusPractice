use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use dioxus_html::geometry::WheelDelta;
use native_dialog::FileDialog;

// const MAIN_CSS: Asset = asset!("/assets/main.css");
// 将 CSS 作为静态字符串包含
static MAIN_CSS: &str = include_str!("../assets/main.css");

fn main() {
    // dioxus::launch(App);
    let config = Config::new()
        .with_menu(None)
        .with_disable_context_menu(true)
        .with_window(
            WindowBuilder::new()
                .with_title("图片查看")
                .with_resizable(true)
                .with_inner_size(dioxus_desktop::LogicalSize::new(800.0, 600.0)),
        );
    dioxus_desktop::launch::launch(
        App,
        Vec::new(),             // contexts
        vec![Box::new(config)], // platform_config
    );
}

fn is_image(path: &std::path::Path) -> bool {
    let img_vec = ["png", "jpg", "jpeg", "gif", "bmp"];

    match path.extension() {
        Some(ext) => img_vec.contains(&ext.to_str().unwrap()),
        None => false,
    }
}

#[component]
fn App() -> Element {
    let mut img_path_list = use_signal(Vec::new);

    let mut current_index = use_signal(|| 0);

    let mut img_src = use_signal(|| "".to_string());
    // 添加缩放比例 state，初始值为 1.0
    let mut scale = use_signal(|| 1.0);
    // 添加拖拽相关状态
    let mut is_dragging = use_signal(|| false);
    let mut start_pos = use_signal(|| (0.0, 0.0));
    let mut offset = use_signal(|| (0.0, 0.0));

    let open = move |_| {
        img_path_list.clear();
        img_path_list().shrink_to_fit();

        if let Ok(Some(path)) = FileDialog::new()
            .set_location("~")
            .add_filter("Pics", &["png", "jpg", "jpeg", "gif", "bmp"])
            .show_open_single_file()
        {
            let dir_path = path.parent().unwrap();
            for entry in dir_path.read_dir().unwrap().flatten() {
                let path = entry.path();
                if is_image(&path) {
                    img_path_list.push(path);
                }
            }
            if let Some(index) = img_path_list().iter().position(|p| p == &path) {
                current_index.set(index);
            }
            scale.set(1.0);
            offset.set((0.0, 0.0));
            img_src.set(path.display().to_string());
        }
    };

    let next = move |_| {
        if img_path_list().is_empty() {
            return;
        }
        let len = img_path_list.len();
        if current_index() == len - 1 {
            current_index.set(0);
        } else {
            current_index += 1;
        }
        let cur_path = img_path_list()[current_index()].display().to_string();
        scale.set(1.0);
        offset.set((0.0, 0.0));
        img_src.set(cur_path);
    };
    let prev = move |_| {
        if img_path_list().is_empty() {
            return;
        }
        let len = img_path_list.len();

        if current_index() == 0 {
            current_index.set(len - 1);
        } else {
            current_index -= 1;
        }
        let cur_path = img_path_list()[current_index()].display().to_string();
        scale.set(1.0);
        offset.set((0.0, 0.0));
        img_src.set(cur_path);
    };

    // 添加滚轮事件处理函数
    let handle_wheel = move |evt: Event<WheelData>| {
        // 处理滚轮事件
        let delta = match evt.data.delta() {
            WheelDelta::Pixels(pixels) => {
                if pixels.y < 0.0 {
                    0.1_f64
                } else {
                    -0.1_f64
                }
            }
            // WheelDelta::Lines(lines) => {
            //     if lines.y < 0.0 { 0.1_f64 } else { -0.1_f64 }
            // }
            // WheelDelta::Pages(pages) => {
            //     if pages.y < 0.0 { 0.1_f64 } else { -0.1_f64 }}
            _ => 1.0_f64,
        };
        // 限制缩放范围，防止过分缩放
        let new_scale = (scale() as f64 + delta).clamp(0.05_f64, 20.0_f64);
        scale.set(new_scale);
    };

    rsx! {
        // document::Link { rel: "stylesheet", href: MAIN_CSS }
        // 使用 style 标签内联 CSS，而不是外部链接
        style { "{MAIN_CSS}" }
        div {
            class: "main-container",
            div {
                class: "img-container",
                onwheel: handle_wheel,
                onmousedown: move |evt| {
                    is_dragging.set(true);
                    start_pos.set((evt.data.client_coordinates().x, evt.data.client_coordinates().y));
                },
                onmousemove: move |evt| {
                    if is_dragging() {
                        let current_x = evt.data.client_coordinates().x;
                        let current_y = evt.data.client_coordinates().y;
                        let (start_x, start_y) = start_pos();
                        let (offset_x, offset_y) = offset();
                        let delta_x = (current_x - start_x) / scale();
                        let delta_y = (current_y - start_y) / scale();
                        offset.set((offset_x + delta_x, offset_y + delta_y));
                        start_pos.set((current_x, current_y));
                    }
                },
                onmouseup: move |_| {
                    is_dragging.set(false);
                },
                onmouseleave: move |_| {
                    is_dragging.set(false);
                },
                img {
                    src: "{img_src}",
                    // 使用 transform: scale() 应用缩放
                    style: "transform: scale({scale}) translate({offset().0}px, {offset().1}px); cursor: normal;"
                }
            }
            div {
                class: "button-container",
                button {
                    onclick: prev,
                    class: "nav-button",
                    "上一张"
                }
                button {
                    onclick: open,
                    class: "nav-button",
                    "打开图片"
                }
                button {
                    onclick: next,
                    class: "nav-button",
                    "下一张"
                }
            }
        }
    }
}
