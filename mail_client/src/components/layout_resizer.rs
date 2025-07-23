use dioxus::prelude::*;
use dioxus_desktop::use_window;

// 用于跟踪拖拽状态的数据
#[derive(Clone, Default)]
pub struct ResizeData {
    pub is_dragging: bool,
    pub element_id: Option<String>,
    pub start_x: f64,
    pub start_width: f64,
    pub container_width: f64,
}

// 使用信号而不是事件处理器，简化状态共享
pub fn use_resize_state() -> (Signal<ResizeData>, Signal<(f64, f64, f64)>) {
    // 创建拖拽状态信号
    let resize_data = use_signal(ResizeData::default);
    
    // 创建保存侧边栏、邮件列表和内容区宽度百分比的信号
    let column_widths = use_signal(|| (20.0, 30.0, 50.0)); // 默认宽度百分比 (侧边栏, 邮件列表, 内容区)
    
    (resize_data, column_widths)
}

#[component]
pub fn ResizeHandle(
    #[props(into)] id: String,
    #[props(into)] resize_data: Signal<ResizeData>,
    #[props(into)] column_widths: Signal<(f64, f64, f64)>,
) -> Element {
    let handle_mouse_down = move |e: MouseEvent| {
        let element_id = id.clone();
        
        // 对于初始宽度，我们使用当前设置的百分比宽度
        let (sidebar_width, email_list_width, content_width) = *column_widths.read();
        
        let initial_width_percent = if element_id == "sidebar" {
            sidebar_width
        } else {
            email_list_width
        };
        
        // 获取窗口的实际宽度
        let window = use_window();
        let container_width = window.inner_size().width as f64;
        
        let initial_width = container_width * initial_width_percent / 100.0;
        
        // 修正: 使用正确的鼠标位置获取方法
        let start_x = e.data.client_coordinates().x;
        
        // 设置拖拽状态
        resize_data.set(ResizeData {
            is_dragging: true,
            element_id: Some(element_id),
            start_x,
            start_width: initial_width,
            container_width,
        });
        
        // 阻止默认行为和冒泡
        e.stop_propagation();
    };
    
    rsx! {
        div {
            class: "resizer",
            onmousedown: handle_mouse_down
        }
    }
}