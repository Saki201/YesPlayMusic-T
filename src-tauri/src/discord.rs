//! Discord Rich Presence（对照 src/electron/ipcMain.js 的 discord-rich-presence 部分）
//!
//! 懒连接：首次播放时尝试连接 Discord IPC，Discord 未运行时静默失败。

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use serde_json::Value;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// 与 Electron 版相同的 Discord Application ID
const APP_ID: &str = "818936529484906596";

static CLIENT: Mutex<Option<DiscordIpcClient>> = Mutex::new(None);

fn with_client(f: impl FnOnce(&mut DiscordIpcClient) -> Result<(), Box<dyn std::error::Error>>) {
    let mut guard = CLIENT.lock().unwrap();
    if guard.is_none() {
        match DiscordIpcClient::new(APP_ID) {
            Ok(mut client) => {
                if client.connect().is_ok() {
                    *guard = Some(client);
                }
            }
            Err(_) => {}
        }
    }
    if let Some(client) = guard.as_mut() {
        if f(client).is_err() {
            // 连接失效（Discord 退出等），重置待下次重连
            *guard = None;
        }
    }
}

fn track_texts(track: &Value) -> (String, String, String) {
    let name = track
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("未知歌曲")
        .to_string();
    let artists = track
        .get("ar")
        .and_then(|v| v.as_array())
        .map(|ar| {
            ar.iter()
                .filter_map(|a| a.get("name").and_then(|n| n.as_str()))
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();
    let album = track
        .get("al")
        .and_then(|al| al.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let pic = track
        .get("al")
        .and_then(|al| al.get("picUrl"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    (format!("{name} - {artists}"), album, pic)
}

/// 播放状态（对照 'playDiscordPresence'）
pub fn play(track: &Value) {
    let (details, state, pic) = track_texts(track);
    let name = track.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let large_text = format!("Listening {name}");
    let dt_ms = track.get("dt").and_then(|v| v.as_i64()).unwrap_or(0);
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    with_client(|client| {
        client.set_activity(
            activity::Activity::new()
                .details(&details)
                .state(&state)
                .timestamps(activity::Timestamps::new().end(now_secs + dt_ms / 1000))
                .assets(
                    activity::Assets::new()
                        .large_image(&pic)
                        .large_text(&large_text)
                        .small_image("play")
                        .small_text("Playing"),
                ),
        )
    });
}

/// 暂停状态（对照 'pauseDiscordPresence'）
pub fn pause(track: &Value) {
    let (details, state, pic) = track_texts(track);

    with_client(|client| {
        client.set_activity(
            activity::Activity::new()
                .details(&details)
                .state(&state)
                .assets(
                    activity::Assets::new()
                        .large_image(&pic)
                        .large_text("YesPlayMusic-T")
                        .small_image("pause")
                        .small_text("Pause"),
                ),
        )
    });
}
