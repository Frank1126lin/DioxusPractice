use dioxus::prelude::*;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use rfd::FileDialog;

fn main() {
    dioxus_desktop::launch(App);
}

#[allow(non_snake_case)]
fn App(cx: Scope) -> Element {
    // 初始化音频状态
    let audio_state = use_ref(cx, || AudioState {
        sink: None,
        is_playing: false,
        volume: 1.0,
        file_path: None,
    });

    // 文件选择逻辑
    let select_file = move |_| {
        if let Some(path) = FileDialog::new()
            .add_filter("Audio", &["mp3", "wav"])
            .pick_file()
        {
            let mut state = audio_state.write();
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            let file = File::open(&path).unwrap();
            let source = Decoder::new(BufReader::new(file)).unwrap();
            sink.append(source);
            sink.pause(); // 初始暂停
            sink.set_volume(state.volume);
            state.sink = Some(Arc::new(Mutex::new(sink)));
            state.file_path = Some(path.display().to_string());
            state.is_playing = false;
        }
    };

    // 播放/暂停切换
    let toggle_play = move |_| {
        let mut state = audio_state.write();
        if let Some(sink) = &state.sink {
            if state.is_playing {
                sink.lock().unwrap().pause();
                state.is_playing = false;
            } else {
                sink.lock().unwrap().play();
                state.is_playing = true;
            }
        }
    };

    // 音量调节
    let update_volume = move |event: FormEvent| {
        let mut state = audio_state.write();
        let new_volume = event.value.parse::<f32>().unwrap_or(1.0).clamp(0.0, 1.0);
        state.volume = new_volume;
        if let Some(sink) = &state.sink {
            sink.lock().unwrap().set_volume(new_volume);
        }
    };

    // 渲染 UI
    cx.render(rsx! {
        div {
            style: "text-align: center; padding: 20px;",
            h1 { "简易音频播放器" }
            button {
                onclick: select_file,
                "选择音频文件"
            }
            div {
                style: "margin-top: 10px;",
                if let Some(path) = &audio_state.read().file_path {
                    p { "当前文件: {path}" }
                } else {
                    p { "未选择文件" }
                }
            }
            if audio_state.read().sink.is_some() {
                rsx! {
                    button {
                        onclick: toggle_play,
                        if audio_state.read().is_playing { "暂停" } else { "播放" }
                    }
                    div {
                        style: "margin-top: 20px;",
                        label { "音量: " }
                        input {
                            r#type: "range",
                            min: "0.0",
                            max: "1.0",
                            step: "0.1",
                            value: "{audio_state.read().volume}",
                            oninput: update_volume
                        }
                        span { "{(audio_state.read().volume * 100.0) as i32}%" }
                    }
                }
            }
        }
    })
}

// 音频状态结构体
#[derive(Clone)]
struct AudioState {
    sink: Option<Arc<Mutex<Sink>>>, // 可选的 Sink，初始为 None
    is_playing: bool,
    volume: f32,
    file_path: Option<String>, // 记录文件路径
}
