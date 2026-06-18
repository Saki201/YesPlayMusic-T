mod discord;
mod ipc;
mod server;
mod settings;
mod shortcuts;
mod tray;
mod unm;
mod update;

use serde_json::Value;
use tauri::{Emitter, Manager};

pub fn run() {
    tauri::Builder::default()
        // 单实例：再次启动时唤起已有窗口（对应 Electron requestSingleInstanceLock + second-instance）
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
                let _ = win.unminimize();
                let _ = win.set_focus();
            }
        }))
        // 窗口大小/位置持久化（对应 Electron 版手写的 store.set('window', bounds)）
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            ipc::ipc_send,
            ipc::update_player_state,
            ipc::log_error,
            unm::unblock_music,
        ])
        .setup(|app| {
            // 全局设置（由前端 'settings' channel 推送，对应 electron-store）
            app.manage(settings::Settings::load(app.handle()));

            // 本地服务：静态资源 + /api（网易云 API）+ /player，端口与 Electron 版一致。
            // 端口绑定失败时直接中止启动，避免窗口加载到其他本地进程暴露的页面。
            if let Err(e) = server::start() {
                eprintln!("[server] {e}");
                return Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, e).into());
            }

            // 主窗口。生产环境加载本地 axum 服务（与 Electron 版加载
            // http://localhost:27232 同构，保证同源 Cookie 行为一致）
            let url: tauri::Url = if cfg!(debug_assertions) {
                "http://localhost:20201".parse().unwrap()
            } else {
                "http://127.0.0.1:27232".parse().unwrap()
            };

            let mut builder = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::External(url),
            )
            .title("YesPlayMusic-T")
            .inner_size(1440.0, 840.0)
            .min_inner_size(1080.0, 720.0)
            // Windows 上隐藏系统标题栏，由前端 Win32Titlebar 组件自绘
            .decorations(false);

            // 持久化的代理设置注入 WebView2 启动参数（运行期无法切换，重启生效）
            let proxy = app
                .state::<settings::Settings>()
                .get_root("proxy")
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default();
            if !proxy.is_empty() {
                builder = builder.additional_browser_args(&format!(
                    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --proxy-server={proxy}"
                ));
            }

            let win = builder.build()?;

            // 最大化状态变化时通知前端（Win32Titlebar 切换最大化/还原图标）
            let w = win.clone();
            win.on_window_event(move |event| match event {
                tauri::WindowEvent::Resized(_) => {
                    let _ = w.emit(
                        "isMaximized",
                        serde_json::json!([w.is_maximized().unwrap_or(false)]),
                    );
                }
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    // Alt+F4 / 系统关闭请求，与标题栏关闭按钮走同一逻辑
                    let app = w.app_handle();
                    if ipc::handle_close(app) {
                        api.prevent_close();
                    }
                }
                _ => {}
            });

            // 系统托盘（Windows 恒启用，对应 platform.js isCreateTray）
            tray::init(app.handle())?;

            // 全局快捷键（对照 background.js:424 enableGlobalShortcut !== false）
            let enable_shortcut = app
                .state::<settings::Settings>()
                .get_setting("enableGlobalShortcut")
                != Some(Value::Bool(false));
            if enable_shortcut {
                shortcuts::register_all(app.handle());
            }

            // 检查更新（GitHub Releases，对照 background.js checkForUpdates）
            update::check(app.handle());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
