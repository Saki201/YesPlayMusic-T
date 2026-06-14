//! UnblockNeteaseMusic 解灰
//!
//! 对照 src/electron/ipcMain.js 的 'unblock-music' handler：
//! 把网易云 track 转为 UNM Song，按 source 列表（默认 ytdl/bilibili/pyncm/kugou）
//! 搜索替代音源并取回播放 URL；bilivideo 链接需带 Referer 拉取后转 base64。

use base64::Engine as _;
use serde_json::Value;
use std::sync::OnceLock;
use unm_engine::executor::Executor;
use unm_types::{config::ConfigManagerBuilder, Album, Artist, Context, SearchMode, Song};

/// 支持的音源（名称 → 引擎注册名），与 Electron 版默认列表一致
const DEFAULT_SOURCES: &[&str] = &["ytdl", "bilibili", "pyncm", "kugou"];

fn executor() -> &'static Executor {
    static EXECUTOR: OnceLock<Executor> = OnceLock::new();
    EXECUTOR.get_or_init(|| {
        use std::borrow::Cow;
        use std::sync::Arc;
        let mut e = Executor::new();
        e.register(
            Cow::Borrowed(unm_engine_bilibili::ENGINE_ID),
            Arc::new(unm_engine_bilibili::BilibiliEngine),
        );
        e.register(
            Cow::Borrowed(unm_engine_kugou::ENGINE_ID),
            Arc::new(unm_engine_kugou::KugouEngine),
        );
        e.register(
            Cow::Borrowed(unm_engine_pyncm::ENGINE_ID),
            Arc::new(unm_engine_pyncm::PyNCMEngine),
        );
        e.register(
            Cow::Borrowed(unm_engine_ytdl::ENGINE_ID),
            Arc::new(unm_engine_ytdl::YtDlEngine),
        );
        e
    })
}

/// 解析 source 字符串（"a, b"）为可用引擎列表（对照 parseSourceStringToList）
fn parse_sources(source_list_string: Option<&str>) -> Vec<String> {
    let available: Vec<String> = executor().list().iter().map(|s| s.to_string()).collect();
    match source_list_string {
        Some(s) if !s.trim().is_empty() => s
            .split(',')
            .map(|x| x.trim().to_lowercase())
            .filter(|x| {
                let ok = available.contains(x);
                if !ok {
                    eprintln!("[unm] 不支持的音源: {x}");
                }
                ok
            })
            .collect(),
        _ => DEFAULT_SOURCES.iter().map(|s| s.to_string()).collect(),
    }
}

/// 网易云 track（{id, name, dt, al, ar}）→ UNM Song（对照 ipcMain.js 的 song 构造）
fn ncm_track_to_song(track: &Value) -> Song {
    let to_string = |v: &Value| match v {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        _ => String::new(),
    };

    let mut song = Song::default();
    song.id = track.get("id").map(&to_string).unwrap_or_default();
    song.name = track
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    song.duration = track.get("dt").and_then(|v| v.as_i64());

    if let Some(al) = track.get("al") {
        let mut album = Album::default();
        album.id = al.get("id").map(&to_string).unwrap_or_default();
        album.name = al
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        song.album = Some(album);
    }

    if let Some(Value::Array(ar)) = track.get("ar") {
        song.artists = ar
            .iter()
            .map(|a| {
                let mut artist = Artist::default();
                artist.id = a.get("id").map(&to_string).unwrap_or_default();
                artist.name = a
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                artist
            })
            .collect();
    }

    song
}

/// 前端 context（{enableFlac, proxyUri, searchMode, config}）→ UNM Context
fn build_context(js: &Value) -> Context {
    let mut ctx = Context::default();

    ctx.enable_flac = js
        .get("enableFlac")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if let Some(proxy) = js.get("proxyUri").and_then(|v| v.as_str()) {
        if !proxy.is_empty() {
            ctx.proxy_uri = Some(std::borrow::Cow::Owned(proxy.to_string()));
        }
    }

    ctx.search_mode = match js.get("searchMode").and_then(|v| v.as_i64()) {
        Some(1) => SearchMode::OrderFirst,
        _ => SearchMode::FastFirst,
    };

    // config：{'joox:cookie', 'qq:cookie', 'ytdl:exe'}，过滤 null 值
    if let Some(Value::Object(config)) = js.get("config") {
        let mut builder = ConfigManagerBuilder::new();
        for (key, value) in config {
            if let Some(v) = value.as_str() {
                if !v.is_empty() {
                    builder.set(key.clone(), v.to_string());
                }
            }
        }
        ctx.config = Some(builder.build());
    }

    ctx
}

/// bilivideo 音频需带 Referer 拉取后转 base64（对照 getBiliVideoFile）
async fn fetch_bili_base64(url: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("Referer", "https://www.bilibili.com/")
        .header("User-Agent", "okhttp/3.4.1")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

#[tauri::command]
pub async fn unblock_music(
    source_list_string: Option<String>,
    ncm_track: Value,
    context: Value,
) -> Value {
    let sources = parse_sources(source_list_string.as_deref());
    if sources.is_empty() {
        return Value::Null;
    }
    eprintln!("[unm] 使用音源: {}", sources.join(", "));

    let song = ncm_track_to_song(&ncm_track);
    let ctx = build_context(&context);

    let executor = executor();
    let source_refs: Vec<std::borrow::Cow<'static, str>> = sources
        .into_iter()
        .map(std::borrow::Cow::Owned)
        .collect();

    let matched = match executor.search(&source_refs, &song, &ctx).await {
        Ok(info) => info,
        Err(e) => {
            eprintln!("[unm] 搜索失败: {e}");
            return Value::Null;
        }
    };

    let mut retrieved = match executor.retrieve(&matched, &ctx).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[unm] 取回音源失败: {e}");
            return Value::Null;
        }
    };

    // bilibili 音源特判：转 base64（前端按 source === 'bilibili' 处理 data URI）
    if retrieved.url.contains("bilivideo.com") {
        match fetch_bili_base64(&retrieved.url).await {
            Ok(b64) => retrieved.url = b64,
            Err(e) => {
                eprintln!("[unm] bilibili 音频拉取失败: {e}");
                return Value::Null;
            }
        }
    }

    serde_json::to_value(&retrieved).unwrap_or(Value::Null)
}
