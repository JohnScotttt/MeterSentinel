// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

/// 需要监听的 HID 设备列表：(vid, pid)。
/// 只要匹配其中任意一项的设备插入，就弹出一个窗口。
struct WatchedDevice {
    vid: u16,
    pid: u16,
    name: &'static str,
}

const WATCH_LIST: &[WatchedDevice] = &[WatchedDevice {
    vid: 0x0716,
    pid: 0x5060,
    name: "WITRN K2",
}];

/// 轮询间隔
const POLL_INTERVAL: Duration = Duration::from_millis(1000);

/// 为已弹出的设备窗口做级联偏移，避免多个窗口完全重叠。
static WINDOW_SEQ: AtomicU32 = AtomicU32::new(0);

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// 根据设备系统路径生成一个稳定且合法的窗口 label。
/// 同一物理设备的路径不变，因此拔插后 label 一致。
fn label_for_path(path: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    format!("hid-{:016x}", hasher.finish())
}

/// 为一个匹配到的 HID 设备创建弹窗（窗内展示设备名与 vid/pid）。
fn open_device_window(app: &tauri::AppHandle, label: &str, vid: u16, pid: u16, name: &str) {
    if app.get_webview_window(label).is_some() {
        return;
    }

    let seq = WINDOW_SEQ.fetch_add(1, Ordering::Relaxed) % 8;
    let offset = (seq * 32) as f64;
    let url = format!(
        "index.html?view=device&vid={:#06x}&pid={:#06x}&name={}",
        vid,
        pid,
        urlencoding::encode(name)
    );

    let _ = WebviewWindowBuilder::new(app, label, WebviewUrl::App(url.into()))
        .title(format!("{} ({:#06x}:{:#06x})", name, vid, pid))
        .inner_size(360.0, 200.0)
        .position(120.0 + offset, 120.0 + offset)
        .resizable(false)
        .decorations(false)
        .visible(false)
        .build();
}

/// 关闭指定设备对应的弹窗（设备移除时调用）。
fn close_device_window(app: &tauri::AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.close();
    }
}

/// 后台线程：轮询 HID 设备列表，按监听列表匹配，插入弹窗、移除关窗。
fn spawn_hid_watcher(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mut api = match hidapi::HidApi::new() {
            Ok(api) => api,
            Err(err) => {
                eprintln!("[hid] 初始化 hidapi 失败: {err}");
                return;
            }
        };

        // 当前已弹窗的设备：path -> (vid, pid, name)
        let mut current: HashMap<String, (u16, u16, &'static str)> = HashMap::new();

        loop {
            if let Err(err) = api.refresh_devices() {
                eprintln!("[hid] 刷新设备列表失败: {err}");
            }

            // 本轮扫描到的、命中监听列表的设备
            let mut found: HashMap<String, (u16, u16, &'static str)> = HashMap::new();
            for dev in api.device_list() {
                let vid = dev.vendor_id();
                let pid = dev.product_id();
                if let Some(watched) =
                    WATCH_LIST.iter().find(|w| w.vid == vid && w.pid == pid)
                {
                    let path = dev.path().to_string_lossy().into_owned();
                    found.insert(path, (vid, pid, watched.name));
                }
            }

            // 新插入的设备 -> 弹窗
            for (path, &(vid, pid, name)) in &found {
                if !current.contains_key(path) {
                    let label = label_for_path(path);
                    let app_handle = app.clone();
                    let _ = app.run_on_main_thread(move || {
                        open_device_window(&app_handle, &label, vid, pid, name);
                    });
                }
            }

            // 已移除的设备 -> 关窗
            for path in current.keys() {
                if !found.contains_key(path) {
                    let label = label_for_path(path);
                    let app_handle = app.clone();
                    let _ = app.run_on_main_thread(move || {
                        close_device_window(&app_handle, &label);
                    });
                }
            }

            current = found;
            std::thread::sleep(POLL_INTERVAL);
        }
    });
}

/// 显示并聚焦主界面
fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 右键菜单项：打开主界面 / 退出
            let open_item = MenuItem::with_id(app, "open", "打开主界面", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_item, &quit_item])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("MeterSentinel")
                .menu(&menu)
                // 只在右键时弹出菜单，避免左键点击也弹菜单
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                // 左键单击托盘图标唤起主界面
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // 启动 HID 设备监听线程
            spawn_hid_watcher(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .on_window_event(|window, event| {
            // 点击关闭按钮时隐藏到托盘，而不是退出应用
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
