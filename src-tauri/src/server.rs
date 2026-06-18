//! 本地 HTTP 服务（127.0.0.1:27232）
//!
//! 1:1 复刻 Electron 版 background.js 的 Express 拓扑：
//! - `/`        前端静态资源（rust-embed 内嵌 dist）
//! - `/api/*`   网易云 API（ncm-api-rs，替代 Node 版 @neteaseapireborn/api）
//! - `/player`  对外播放状态接口（供桌面歌词等第三方工具查询）

use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use rust_embed::RustEmbed;
use serde_json::Value;
use std::{net::ToSocketAddrs, sync::Mutex};

const SERVER_ADDR: &str = "127.0.0.1:27232";

/// 前端构建产物（beforeBuildCommand 先于 cargo 编译执行，保证 dist 已生成）
#[derive(RustEmbed)]
#[folder = "../dist/"]
struct Dist;

/// 当前播放状态缓存，由前端每秒推送（tauriBridge.js → update_player_state 命令）
pub static PLAYER_STATE: Mutex<Value> = Mutex::new(Value::Null);

async fn player_handler() -> Json<Value> {
    Json(PLAYER_STATE.lock().unwrap().clone())
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Dist::get(path) {
        Some(file) => (
            [(header::CONTENT_TYPE, content_type(path))],
            file.data,
        )
            .into_response(),
        // SPA 路由（无扩展名，如直接访问 /library）回退到 index.html
        None if is_spa_route(path) => match Dist::get("index.html") {
            Some(file) => (
                [(header::CONTENT_TYPE, "text/html; charset=utf-8".to_string())],
                file.data,
            )
                .into_response(),
            None => (StatusCode::NOT_FOUND, "前端资源未构建").into_response(),
        },
        // 带扩展名的资源（/js/*.js、/css/*.css 等）未命中直接 404。
        // 旧实现统一回退 index.html：浏览器把 HTML 当 JS 解析立即抛语法错误，
        // 在 release（windows_subsystem 无控制台）下表现为白屏。
        None => (StatusCode::NOT_FOUND, "资源不存在").into_response(),
    }
}

/// 按扩展名返回 Content-Type。.js/.css 显式指定，避免 mime_guess 版本差异
/// （不同 feature 下 .js 可能返回 application/javascript 或 text/javascript）。
fn content_type(path: &str) -> String {
    match path.rsplit('.').next() {
        Some("js") | Some("mjs") => "text/javascript; charset=utf-8".to_string(),
        Some("css") => "text/css; charset=utf-8".to_string(),
        Some("html") | Some("htm") => "text/html; charset=utf-8".to_string(),
        Some("json") => "application/json; charset=utf-8".to_string(),
        Some("svg") => "image/svg+xml".to_string(),
        Some("woff") => "font/woff".to_string(),
        Some("woff2") => "font/woff2".to_string(),
        Some("png") => "image/png".to_string(),
        Some("ico") => "image/x-icon".to_string(),
        Some("webmanifest") => "application/manifest+json".to_string(),
        _ => mime_guess::from_path(path)
            .first_or_octet_stream()
            .as_ref()
            .to_string(),
    }
}

/// 是否为 SPA 路由路径（无文件扩展名），用于决定是否回退 index.html。
/// /api、/player 由上游 nest/route 处理，不会走到这里。
fn is_spa_route(path: &str) -> bool {
    !path.contains('.')
}

/// 登录接口响应兼容层：抹平 ncm-api-rs 与 Node 版 NeteaseCloudMusicApi 的结构差异
///
/// 1. Node 版登录系模块会把 Set-Cookie 以 `';;'` 连接塞进响应体 `cookie` 字段，
///    前端 setCookies() 依赖它持久化 MUSIC_U（utils/auth.js）
/// 2. Node 版 /login/qr/key、/login/qr/create 会把原始响应包进 `{code, data}` 一层，
///    前端取 `result.data.unikey`（loginAccount.vue）
async fn login_compat(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let is_login = path.starts_with("/login") || path.starts_with("/logout");
    let resp = next.run(req).await;
    if !is_login {
        return resp;
    }

    let cookies: Vec<String> = resp
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok().map(String::from))
        .collect();

    let (mut parts, body) = resp.into_parts();
    let bytes = match axum::body::to_bytes(body, 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::BAD_GATEWAY, "login_compat 读取响应失败").into_response(),
    };

    let mut json: Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        // 非 JSON 响应原样透传
        Err(_) => return Response::from_parts(parts, Body::from(bytes)),
    };

    if !cookies.is_empty() && json.get("cookie").is_none() {
        json["cookie"] = Value::String(cookies.join(";;"));
    }

    if path == "/login/qr/key" || path == "/login/qr/create" {
        json = serde_json::json!({ "code": 200, "data": json });
    }

    let new_bytes = serde_json::to_vec(&json).unwrap_or_else(|_| bytes.to_vec());
    parts.headers.remove(header::CONTENT_LENGTH);
    Response::from_parts(parts, Body::from(new_bytes))
}

fn bind_std_listener(addr: impl ToSocketAddrs) -> Result<std::net::TcpListener, String> {
    let listener = std::net::TcpListener::bind(addr).map_err(|e| format!("端口绑定失败: {e}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("设置非阻塞监听失败: {e}"))?;
    Ok(listener)
}

#[cfg(test)]
fn bind_listener_for_test(
    addr: std::net::SocketAddr,
) -> Result<std::net::TcpListener, String> {
    bind_std_listener(addr)
}

pub fn start() -> Result<(), String> {
    // 纯 std 绑定：不依赖 Tokio reactor，可在 setup() 同步上下文里安全调用。
    // 端口冲突在此被 ? 捕获，中止启动（与原行为一致）。
    let std_listener = bind_std_listener(SERVER_ADDR)?;

    tauri::async_runtime::spawn(async move {
        // from_std 需在 Tokio 运行时上下文内调用。debug 构建下 setup 主线程
        // 可能恰好命中 reactor，但 release（单线程 runtime）主线程无 reactor，
        // 在 setup 同步上下文调 from_std 会 panic：
        //   "there is no reactor running, must be called from the context of a Tokio 1.x runtime"
        // 表现为安装版双击无反应（进程在 setup 阶段 panic 静默退出）。
        // 搬进 spawn 的 async 块即处于 reactor 上下文，问题消除。
        let listener = match tokio::net::TcpListener::from_std(std_listener) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[server] 创建 tokio 监听器失败: {e}");
                return;
            }
        };

        let ncm = ncm_api_rs::server::build_app(ncm_api_rs::create_client(None))
            .layer(middleware::from_fn(login_compat));

        let router = Router::new()
            .route("/player", get(player_handler))
            .nest("/api", ncm)
            .fallback(static_handler);

        println!("[server] listening on http://{SERVER_ADDR}");
        if let Err(e) = axum::serve(listener, router).await {
            eprintln!("[server] 服务异常退出: {e}");
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::bind_listener_for_test;

    #[test]
    fn bind_listener_reports_port_conflict() {
        let occupied = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = occupied.local_addr().unwrap();

        let err = bind_listener_for_test(addr).expect_err("端口占用时应返回错误");

        assert!(
            err.contains("绑定失败") || err.contains("Address already in use"),
            "错误信息应说明绑定失败，实际为: {err}"
        );
    }
}
