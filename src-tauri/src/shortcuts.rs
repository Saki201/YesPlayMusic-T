//! 全局快捷键（对照 src/electron/globalShortcut.js）
//!
//! 默认值与 src/utils/shortcuts.js 一致；用户自定义存在 settings.shortcuts。
//! Electron accelerator 格式（CommandOrControl+Left）需转换为
//! global_hotkey 可解析的格式（Control+ArrowLeft）。

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::settings::Settings;

/// 默认快捷键（id, globalShortcut），与 src/utils/shortcuts.js 同步
const DEFAULTS: &[(&str, &str)] = &[
    ("play", "Alt+CommandOrControl+P"),
    ("next", "Alt+CommandOrControl+Right"),
    ("previous", "Alt+CommandOrControl+Left"),
    ("increaseVolume", "Alt+CommandOrControl+Up"),
    ("decreaseVolume", "Alt+CommandOrControl+Down"),
    ("like", "Alt+CommandOrControl+L"),
    ("minimize", "Alt+CommandOrControl+M"),
];

/// 完整默认快捷键表（restoreDefaultShortcuts 用），与 src/utils/shortcuts.js 一致
pub fn default_shortcuts_json() -> Value {
    serde_json::json!([
        { "id": "play", "name": "播放/暂停", "shortcut": "CommandOrControl+P", "globalShortcut": "Alt+CommandOrControl+P" },
        { "id": "next", "name": "下一首", "shortcut": "CommandOrControl+Right", "globalShortcut": "Alt+CommandOrControl+Right" },
        { "id": "previous", "name": "上一首", "shortcut": "CommandOrControl+Left", "globalShortcut": "Alt+CommandOrControl+Left" },
        { "id": "increaseVolume", "name": "增加音量", "shortcut": "CommandOrControl+Up", "globalShortcut": "Alt+CommandOrControl+Up" },
        { "id": "decreaseVolume", "name": "减少音量", "shortcut": "CommandOrControl+Down", "globalShortcut": "Alt+CommandOrControl+Down" },
        { "id": "like", "name": "喜欢歌曲", "shortcut": "CommandOrControl+L", "globalShortcut": "Alt+CommandOrControl+L" },
        { "id": "minimize", "name": "隐藏/显示播放器", "shortcut": "CommandOrControl+M", "globalShortcut": "Alt+CommandOrControl+M" }
    ])
}

/// Electron accelerator → global_hotkey 格式
///
/// 修饰键：CommandOrControl/CmdOrCtrl/Command/Cmd → Control（仅 Windows）
/// 键名：单字母/数字直接透传（global_hotkey 支持 "P"、"1"），
///       方向键 Up/Down/Left/Right → ArrowUp/...，
///       常见符号映射到 W3C code 名。
fn to_hotkey_format(accel: &str) -> String {
    accel
        .split('+')
        .map(|part| match part {
            "CommandOrControl" | "CmdOrCtrl" | "Command" | "Cmd" => "Control".to_string(),
            "Up" => "ArrowUp".to_string(),
            "Down" => "ArrowDown".to_string(),
            "Left" => "ArrowLeft".to_string(),
            "Right" => "ArrowRight".to_string(),
            "=" => "Equal".to_string(),
            "-" => "Minus".to_string(),
            "~" => "Backquote".to_string(),
            "[" => "BracketLeft".to_string(),
            "]" => "BracketRight".to_string(),
            ";" => "Semicolon".to_string(),
            "'" => "Quote".to_string(),
            "," => "Comma".to_string(),
            "." => "Period".to_string(),
            "/" => "Slash".to_string(),
            other => other.to_string(),
        })
        .collect::<Vec<_>>()
        .join("+")
}

/// 从 settings 读取快捷键表（未配置时用默认值），返回 (id, globalShortcut) 列表
fn shortcut_list(app: &AppHandle) -> Vec<(String, String)> {
    let stored = app.state::<Settings>().get_setting("shortcuts");
    if let Some(Value::Array(items)) = stored {
        let list: Vec<(String, String)> = items
            .iter()
            .filter_map(|item| {
                let id = item.get("id")?.as_str()?.to_string();
                let gs = item.get("globalShortcut")?.as_str()?.to_string();
                Some((id, gs))
            })
            .collect();
        if !list.is_empty() {
            return list;
        }
    }
    DEFAULTS
        .iter()
        .map(|(id, gs)| (id.to_string(), gs.to_string()))
        .collect()
}

/// 触发快捷键对应的行为
fn dispatch(app: &AppHandle, id: &str) {
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    match id {
        // minimize：切换窗口显示/隐藏（对照 globalShortcut.js 的 win.isVisible() 分支）
        "minimize" => {
            if win.is_visible().unwrap_or(false) {
                let _ = win.hide();
            } else {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }
        channel => {
            let _ = win.emit(channel, serde_json::json!([]));
        }
    }
}

pub fn register_all(app: &AppHandle) {
    let gs = app.global_shortcut();
    // 防御：先清理可能的残留注册（global-hotkey 在某些场景会保留陈旧条目）
    let _ = gs.unregister_all();

    // 同一进程内对解析后的热键去重，避免 global-hotkey 内部去重表报 "already registered"
    let mut seen = std::collections::HashSet::new();
    for (id, accel) in shortcut_list(app) {
        let hotkey = to_hotkey_format(&accel);
        if !seen.insert(hotkey.clone()) {
            eprintln!("[shortcuts] 跳过重复热键 {id} ({hotkey})");
            continue;
        }
        let id_owned = id.clone();
        let result = gs.on_shortcut(hotkey.as_str(), move |app, _shortcut, event| {
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                dispatch(app, &id_owned);
            }
        });
        if let Err(e) = result {
            eprintln!("[shortcuts] 注册失败 {id} ({accel} → {hotkey}): {e}");
        }
    }
}

pub fn unregister_all(app: &AppHandle) {
    let _ = app.global_shortcut().unregister_all();
}

pub fn re_register(app: &AppHandle) {
    unregister_all(app);
    register_all(app);
}
