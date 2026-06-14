# YesPlayMusic-T Tauri 版（Windows）

YesPlayMusic-T 是 Electron → Tauri 2 重构的 Windows 桌面版。Electron 代码原样保留（双轨并存），
本文档只覆盖 Tauri 相关内容。

## 环境要求

- Node.js（任意新版，构建脚本已注入 `--openssl-legacy-provider` 兼容旧版 webpack 4）
- pnpm
- Rust stable（MSVC 工具链）+ Visual Studio C++ 构建工具
- WebView2 Runtime（Windows 11 自带）

## 常用命令

```bash
pnpm install                # 安装依赖（.npmrc 已配置 shamefully-hoist 兼容旧工具链）
pnpm tauri:dev              # 开发模式（自动起 Vue dev server + Rust 编译 + 打开窗口）
pnpm tauri:build            # 产出 NSIS 安装包（src-tauri/target/release/bundle/nsis/）
```

## 架构

1:1 复刻 Electron 版拓扑，前端代码几乎零改动：

| Electron 版 | Tauri 版 |
|---|---|
| `@neteaseapireborn/api`（Node，端口 10754） | `ncm-api-rs` crate（Rust 原生，挂在 axum `/api` 下） |
| Express（27232：静态 + /api 代理 + /player） | axum（27232：rust-embed 静态 + /api + /player） |
| 窗口加载 `http://localhost:27232` | 同（同源 Cookie 行为完全一致） |
| `ipcRenderer`（34 处调用） | `src/utils/tauriBridge.js` 垫片（`window.require` 兼容层），前端零改动 |
| `@unblockneteasemusic/rust-napi` | 底层 `unm_engine` 系列 crate 直接集成 |
| electron-store | `%APPDATA%/com.qier222.yesplaymusic-t/settings.json` |
| electron-updater + 弹窗开 Releases 页 | GitHub API 查版本 + 弹窗开 Releases 页（无需签名设施） |

关键文件：

- `src-tauri/src/lib.rs` —— 应用入口：窗口、插件、托盘、快捷键、更新检查
- `src-tauri/src/server.rs` —— 本地 27232 服务
- `src-tauri/src/ipc.rs` —— 所有 ipcRenderer channel 的分发中心
- `src-tauri/src/unm.rs` —— 解灰（默认音源 ytdl/bilibili/pyncm/kugou）
- `src/utils/tauriBridge.js` —— 前端桥（必须保持为 main.js 第一个 import）
- `vue.config.js` —— `TAURI_BUILD=1` 时注入 `IS_ELECTRON/IS_TAURI/process.platform` 编译期常量

## 与 Electron 版的已知差异

- **关闭询问对话框**无"记住我的选择"勾选框（可在设置页选择关闭行为）；Esc 等同"直接退出"
- **Last.fm 授权**弹窗流程不可用（Electron 为其开子窗口；Tauri 下在系统浏览器打开，回调无法回到应用）；其余外链正常走系统浏览器
- **代理设置**：API 请求代理即时生效（随请求参数下发）；WebView2 媒体流量代理需重启应用生效
- **解灰音源**注册了 ytdl/bilibili/pyncm/kugou 四个默认源（qq/joox/kuwo/migu 未注册）；ytdl 源需要系统安装 yt-dlp
- macOS（Touch Bar/dock）、Linux（MPRIS/osdlyrics）专属功能不适用，相关 IPC 为 no-op

## 上游依赖修正记录

- `plyr` 钉死 3.7.8（3.8+ 仅有 `exports` 字段，webpack 4 无法解析）
- `winapi` 显式启用 `winbase` feature（修复 `unm_engine_ytdl` 0.4.0 在 Windows 的上游 bug）
