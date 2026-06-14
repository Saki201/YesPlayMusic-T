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
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                [(header::CONTENT_TYPE, mime.as_ref().to_string())],
                file.data,
            )
                .into_response()
        }
        // 前端使用 hash 路由，未命中的路径回退到 index.html
        None => match Dist::get("index.html") {
            Some(file) => (
                [(header::CONTENT_TYPE, "text/html".to_string())],
                file.data,
            )
                .into_response(),
            None => (StatusCode::NOT_FOUND, "前端资源未构建").into_response(),
        },
    }
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
    let listener = bind_std_listener(SERVER_ADDR)?;
    let listener = tokio::net::TcpListener::from_std(listener)
        .map_err(|e| format!("创建 tokio 监听器失败: {e}"))?;

    tauri::async_runtime::spawn(async move {
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
