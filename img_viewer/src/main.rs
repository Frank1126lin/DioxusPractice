use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use native_dialog::FileDialog;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    // dioxus::launch(App);
    let config = Config::new().with_menu(None).with_window(
        WindowBuilder::new()
            .with_title("ImageViewer")
            .with_resizable(true),
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
        Some(ext) => {img_vec.contains(&ext.to_str().unwrap())}
        None => false,
    }
}

#[component]
fn App() -> Element {
    let mut img_path_list = use_signal(Vec::new);

    let mut current_index = use_signal(|| 0);

    let mut img_src = use_signal(|| "".to_string());

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
        img_src.set(cur_path);
    };

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div {
            id: "contaner",
            img {
                id: "img",
                src: "{img_src}",
                max_height: "1000px",
            }
        }
        div {
            style: "display: flex; gap: 25px;", // 使用 flex 布局并设置间距

            button {
                id: "button",
                onclick: prev,
                id: "prev",
                width: "100px",
                height: "100px",
                font_size: "20px",
                "上一张"
            }
            button {
                id: "button",
                onclick: open,
                id: "open",
                width: "100px",
                height: "100px",
                font_size: "20px",
                "打开图片"
            }

            button {
                id: "button",
                onclick: next,
                id: "next",
                width: "100px",
                height: "100px",
                font_size: "20px",
                "下一张"
            }
        }

    }
}
