//! IPC 调度中心
//!
//! 前端 tauriBridge.js 把所有 Electron `ipcRenderer.send(channel, ...args)`
//! 转发为 `ipc_send` 命令，这里按 channel 分发，对照 src/electron/ipcMain.js。
//!
//! Linux/macOS 专用 channel（seeked、metadata、sendLyrics 等 mpris 系）直接 no-op。

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

use crate::settings::Settings;
use crate::{shortcuts, tray};

/// 处理关闭请求（标题栏关闭按钮 + Alt+F4 共用）。
/// 返回 true 表示阻止默认关闭。
///
/// 对照 ipcMain.js 的 'close' handler：
/// - exit            → 直接退出
/// - minimizeToTray  → 隐藏窗口
/// - 其他（未设置）  → 弹询问对话框（回车=最小化到托盘，"直接退出"按钮退出）
pub fn handle_close(app: &AppHandle) -> bool {
    let close_opt = app
        .state::<Settings>()
        .get_setting_str("closeAppOption")
        .unwrap_or_default();

    match close_opt.as_str() {
        "exit" => {
            app.exit(0);
            false
        }
        "minimizeToTray" => {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.hide();
            }
            true
        }
        _ => {
            let app2 = app.clone();
            app.dialog()
                .message("确定要关闭吗？")
                .title("Information")
                .buttons(MessageDialogButtons::OkCancelCustom(
                    "最小化到托盘".into(),
                    "直接退出".into(),
                ))
                .show(move |minimize| {
                    if minimize {
                        if let Some(win) = app2.get_webview_window("main") {
                            let _ = win.hide();
                        }
                    } else {
                        app2.exit(0);
                    }
                });
            true
        }
    }
}

#[tauri::command]
pub async fn ipc_send(
    app: AppHandle,
    window: WebviewWindow,
    channel: String,
    args: Vec<Value>,
) -> Result<Value, String> {
    match channel.as_str() {
        // ---- 窗口控制（Win32Titlebar.vue）----
        "minimize" => {
            let _ = window.minimize();
        }
        "maximizeOrUnmaximize" => {
            let maximized = window.is_maximized().unwrap_or(false);
            if maximized {
                let _ = window.unmaximize();
            } else {
                let _ = window.maximize();
            }
            let _ = window.emit("isMaximized", serde_json::json!([!maximized]));
        }
        "fullscreen" => {
            let fullscreen = window.is_fullscreen().unwrap_or(false);
            let _ = window.set_fullscreen(!fullscreen);
            let _ = window.emit("fullscreenChanged", serde_json::json!([!fullscreen]));
        }
        "close" => {
            handle_close(&app);
        }

        // ---- 设置同步（store/plugins/sendSettings.js → ipcMain.js 'settings'）----
        "settings" => {
            if let Some(settings) = args.first() {
                let enable_shortcut = settings
                    .get("enableGlobalShortcut")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                app.state::<Settings>().update_settings(settings.clone());
                if enable_shortcut {
                    shortcuts::re_register(&app);
                } else {
                    shortcuts::unregister_all(&app);
                }
            }
        }

        // ---- 托盘（Player.js / settings.vue）----
        "updateTrayTooltip" => {
            if let Some(title) = args.first().and_then(|v| v.as_str()) {
                tray::set_tooltip(title);
            }
        }
        "updateTrayPlayState" => {
            if let Some(playing) = args.first().and_then(|v| v.as_bool()) {
                tray::set_playing(&app, playing);
            }
        }
        "updateTrayLikeState" => {
            if let Some(liked) = args.first().and_then(|v| v.as_bool()) {
                tray::set_liked(&app, liked);
            }
        }
        "updateTrayIcon" => {
            tray::update_icon(&app);
        }

        // ---- 代理（settings.vue）----
        // API 请求的代理由前端 request.js 以 proxy 参数逐请求携带（ncm-api-rs 支持）；
        // 这里持久化代理配置，供下次启动时注入 WebView2 启动参数（媒体/图片流量）。
        "setProxy" => {
            if let Some(config) = args.first() {
                let protocol = config.get("protocol").and_then(|v| v.as_str()).unwrap_or("");
                let server = config.get("server").and_then(|v| v.as_str()).unwrap_or("");
                let port = config
                    .get("port")
                    .map(|v| match v {
                        Value::Number(n) => n.to_string(),
                        Value::String(s) => s.clone(),
                        _ => String::new(),
                    })
                    .unwrap_or_default();
                let rules = format!("{protocol}://{server}:{port}");
                app.state::<Settings>().set_root("proxy", Value::String(rules));
            }
        }
        "removeProxy" => {
            app.state::<Settings>()
                .set_root("proxy", Value::String(String::new()));
        }

        // ---- 全局快捷键（settings.vue）----
        "switchGlobalShortcutStatusTemporary" => {
            match args.first().and_then(|v| v.as_str()) {
                Some("disable") => shortcuts::unregister_all(&app),
                _ => shortcuts::re_register(&app),
            }
        }
        "updateShortcut" => {
            // payload: { id, type: 'shortcut' | 'globalShortcut', shortcut }
            if let Some(payload) = args.first() {
                let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let kind = payload.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let accel = payload
                    .get("shortcut")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !id.is_empty() && !kind.is_empty() {
                    app.state::<Settings>().update_shortcut(id, kind, accel);
                    shortcuts::re_register(&app);
                }
            }
        }
        "restoreDefaultShortcuts" => {
            app.state::<Settings>()
                .set_setting("shortcuts", shortcuts::default_shortcuts_json());
            shortcuts::re_register(&app);
        }

        // ---- Discord RPC ----
        "playDiscordPresence" => {
            if let Some(track) = args.first().cloned() {
                tauri::async_runtime::spawn_blocking(move || crate::discord::play(&track));
            }
        }
        "pauseDiscordPresence" => {
            if let Some(track) = args.first().cloned() {
                tauri::async_runtime::spawn_blocking(move || crate::discord::pause(&track));
            }
        }

        // ---- 外部链接（对照 background.js new-window → shell.openExternal）----
        "openExternal" => {
            if let Some(url) = args.first().and_then(|v| v.as_str()) {
                use tauri_plugin_opener::OpenerExt;
                let _ = app.opener().open_url(url, None::<&str>);
            }
        }

        // ---- Linux mpris / macOS 专用，Windows 版 no-op ----
        "player" | "seeked" | "metadata" | "sendLyrics" | "playerCurrentTrackTime"
        | "switchRepeatMode" | "switchShuffle" => {}

        other => {
            #[cfg(debug_assertions)]
            eprintln!("[ipc] 未处理的 channel: {other}, args: {args:?}");
            let _ = other;
        }
    }

    Ok(Value::Null)
}

/// 前端启动期未捕获错误落盘（对照 tauriBridge.js 的 error/unhandledrejection 监听）。
/// release 为 windows_subsystem 无控制台，借此把排查线索留存到
/// `%APPDATA%/com.qier222.yesplaymusic-t/error.log`。
#[tauri::command]
pub fn log_error(app: AppHandle, message: String) {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    let Some(dir) = app.path().app_config_dir().ok() else {
        return;
    };
    let _ = std::fs::create_dir_all(&dir);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("error.log"))
    {
        let _ = writeln!(f, "[{stamp}] {message}");
    }
}

/// 前端每秒推送播放状态，缓存给 /player 对外接口
#[tauri::command]
pub fn update_player_state(state: Value) {
    *crate::server::PLAYER_STATE.lock().unwrap() = state;
}
