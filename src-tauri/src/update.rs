//! 自动检查更新（对照 background.js 的 checkForUpdates）
//!
//! 与 Electron 版 UX 一致：查询 GitHub 最新 Release，发现新版本时弹窗，
//! 用户确认后打开 Releases 页面下载（不做应用内静默更新，无需签名设施）。

use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_opener::OpenerExt;

const RELEASES_URL: &str = "https://github.com/Saki201/YesPlayMusic-T/releases";
const LATEST_API: &str = "https://api.github.com/repos/Saki201/YesPlayMusic-T/releases/latest";

pub fn check(app: &AppHandle) {
    // 开发模式跳过（对照 isDevelopment return）
    if cfg!(debug_assertions) {
        return;
    }

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let current = app.package_info().version.clone();

        let Ok(resp) = reqwest::Client::new()
            .get(LATEST_API)
            .header("User-Agent", "YesPlayMusic-T")
            .send()
            .await
        else {
            return;
        };
        let Ok(json) = resp.json::<serde_json::Value>().await else {
            return;
        };
        let Some(tag) = json.get("tag_name").and_then(|v| v.as_str()) else {
            return;
        };
        let Ok(latest) = semver::Version::parse(tag.trim_start_matches('v')) else {
            return;
        };

        if latest > current {
            let app2 = app.clone();
            app.dialog()
                .message(format!("发现新版本 v{latest}，是否前往 GitHub 下载新版本安装包？"))
                .title(format!("发现新版本 v{latest}"))
                .buttons(MessageDialogButtons::OkCancelCustom(
                    "下载".into(),
                    "取消".into(),
                ))
                .show(move |download| {
                    if download {
                        let _ = app2.opener().open_url(RELEASES_URL, None::<&str>);
                    }
                });
        }
    });
}
