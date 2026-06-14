//! 应用设置持久化（对应 Electron 版 electron-store）
//!
//! 前端在每次 settings 变更时通过 'settings' channel 推送完整 settings 对象，
//! 这里落盘到 `%APPDATA%/com.qier222.yesplaymusic-t/settings.json`。
//! 文件结构镜像 electron-store：`{ "settings": {...}, "proxy": "..." }`

use serde_json::Value;
use std::{fs, path::PathBuf, sync::Mutex};
use tauri::Manager;

pub struct Settings {
    data: Mutex<Value>,
    path: PathBuf,
}

impl Settings {
    pub fn load(app: &tauri::AppHandle) -> Self {
        let dir = app
            .path()
            .app_config_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("settings.json");

        let data = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::json!({}));

        Self {
            data: Mutex::new(data),
            path,
        }
    }

    fn save(&self, data: &Value) {
        if let Ok(text) = serde_json::to_string_pretty(data) {
            let _ = fs::write(&self.path, text);
        }
    }

    /// 更新 settings 子对象（前端推送的 state.settings）
    pub fn update_settings(&self, settings: Value) {
        let mut data = self.data.lock().unwrap();
        data["settings"] = settings;
        self.save(&data);
    }

    /// 读取 settings.<key> 字符串值
    pub fn get_setting_str(&self, key: &str) -> Option<String> {
        self.data
            .lock()
            .unwrap()
            .get("settings")
            .and_then(|s| s.get(key))
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    /// 读取 settings.<key> 原始值
    #[allow(dead_code)]
    pub fn get_setting(&self, key: &str) -> Option<Value> {
        self.data
            .lock()
            .unwrap()
            .get("settings")
            .and_then(|s| s.get(key))
            .cloned()
    }

    /// 读取/写入顶层键（如 proxy）
    #[allow(dead_code)]
    pub fn get_root(&self, key: &str) -> Option<Value> {
        self.data.lock().unwrap().get(key).cloned()
    }

    pub fn set_root(&self, key: &str, value: Value) {
        let mut data = self.data.lock().unwrap();
        data[key] = value;
        self.save(&data);
    }

    /// 写入 settings.<key>
    pub fn set_setting(&self, key: &str, value: Value) {
        let mut data = self.data.lock().unwrap();
        if !data.get("settings").map(|s| s.is_object()).unwrap_or(false) {
            data["settings"] = serde_json::json!({});
        }
        data["settings"][key] = value;
        self.save(&data);
    }

    /// 修改 settings.shortcuts 中指定 id 的某个字段
    /// （对照 ipcMain.js 'updateShortcut'：kind 为 'shortcut' 或 'globalShortcut'）
    pub fn update_shortcut(&self, id: &str, kind: &str, accel: &str) {
        let mut data = self.data.lock().unwrap();
        let shortcuts = data
            .get_mut("settings")
            .and_then(|s| s.get_mut("shortcuts"))
            .and_then(|v| v.as_array_mut());
        if let Some(items) = shortcuts {
            for item in items.iter_mut() {
                if item.get("id").and_then(|v| v.as_str()) == Some(id) {
                    item[kind] = Value::String(accel.to_string());
                    break;
                }
            }
            self.save(&data);
        }
    }
}
