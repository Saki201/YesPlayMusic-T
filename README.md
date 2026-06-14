# YesPlayMusic-T

YesPlayMusic-T 是面向 Windows 的 YesPlayMusic 重构版。项目保留原有 Vue 2 前端体验，将桌面运行时从 Electron 迁移到 Tauri 2，并补齐当前重构目标中的评论面板、音质切换、Windows 托盘、全局快捷键和本地播放状态接口。

本仓库当前目标只保证 Windows 端可用。macOS Touch Bar、dock、Linux MPRIS/osdlyrics 等旧 Electron 跨平台能力不属于当前发布范围。

## 项目状态

- 桌面端主线：Tauri 2 + Rust + WebView2
- 前端主线：Vue 2 + Vue CLI 4 / Webpack 4
- 包管理器：pnpm
- 目标平台：Windows 10/11 x64
- 安装包格式：NSIS

Electron 代码仍保留在仓库中，主要用于对照迁移和保留旧实现参考。当前 Windows 桌面发布应优先使用 Tauri 命令。

## 功能范围

- 网易云账号登录：扫码、手机、邮箱登录
- 播放器基础能力：播放、暂停、上一首、下一首、播放队列、私人 FM
- 歌词页：歌词显示、翻译、罗马音、背景色提取、全屏歌词
- 评论面板：歌曲评论、排序、楼层回复、点赞、发送和回复评论
- 音质切换：128K、192K、320K、FLAC、Hi-Res，并在切换后刷新当前歌曲音源缓存
- 解灰：集成 UnblockNeteaseMusic Rust 引擎，默认音源为 ytdl、bilibili、pyncm、kugou
- 桌面能力：自绘 Windows 标题栏、系统托盘、全局快捷键、关闭到托盘、Discord Rich Presence
- 本地接口：`127.0.0.1:27232/player` 提供当前播放状态

## 架构概览

Tauri 版尽量复刻旧 Electron 版拓扑，让前端改动保持可控：

| 职责 | Electron 旧实现 | Tauri 当前实现 |
|---|---|---|
| 桌面窗口 | Electron `BrowserWindow` | Tauri `WebviewWindow` |
| 本地静态服务 | Express `127.0.0.1:27232` | axum `127.0.0.1:27232` |
| 网易云 API | `@neteaseapireborn/api` Node 服务 | `ncm-api-rs` Rust 服务 |
| 前端 IPC | `window.require('electron').ipcRenderer` | `src/utils/tauriBridge.js` 兼容层 |
| 设置持久化 | `electron-store` | `%APPDATA%/com.qier222.yesplaymusic-t/settings.json` |
| 解灰 | `@unblockneteasemusic/rust-napi` | `unm_engine` 系列 Rust crate |
| 更新检查 | `electron-updater` | GitHub Releases 检查并打开下载页 |

关键文件：

- `src/main.js`：前端入口，必须最先加载 `src/utils/tauriBridge.js`
- `src/utils/tauriBridge.js`：Tauri 下的 Electron IPC 兼容层
- `src/utils/Player.js`：播放器核心、音源选择、缓存、解灰、Discord 状态
- `src/views/lyrics.vue`：歌词页、评论面板、音质切换入口
- `src-tauri/src/lib.rs`：Tauri 应用入口、窗口、托盘、快捷键、更新检查
- `src-tauri/src/server.rs`：本地 HTTP 服务、静态资源、网易云 API、`/player`
- `src-tauri/src/ipc.rs`：前端 IPC channel 分发中心
- `src-tauri/src/unm.rs`：UnblockNeteaseMusic Rust 集成

## 环境要求

- Windows 10/11
- Node.js 16 或兼容旧 Vue CLI 4 工具链的 Node 版本
- pnpm
- Rust stable MSVC 工具链
- Visual Studio C++ Build Tools
- WebView2 Runtime

Windows 11 通常已自带 WebView2 Runtime。构建 NSIS 安装包需要完整 Rust/MSVC 编译环境。

## 开发命令

安装依赖：

```bash
pnpm install
```

启动 Tauri 开发版：

```bash
pnpm tauri:dev
```

构建 Windows 安装包：

```bash
pnpm tauri:build
```

构建产物位置：

```text
src-tauri/target/release/bundle/nsis/
```

仅启动 Web 前端：

```bash
pnpm serve
```

运行旧 Electron 开发版：

```bash
pnpm electron:serve
```

Electron 相关命令仍可用于对照旧行为，但不是当前 Windows 重构版的主发布路径。

## 配置说明

常用环境变量位于 `.env.example`：

```text
VUE_APP_NETEASE_API_URL=/api
VUE_APP_ELECTRON_API_URL=/api
VUE_APP_ELECTRON_API_URL_DEV=http://127.0.0.1:10754
VUE_APP_LASTFM_API_KEY=...
VUE_APP_LASTFM_API_SHARED_SECRET=...
DEV_SERVER_PORT=20201
```

Tauri 开发和构建命令会设置 `TAURI_BUILD=1`，并通过 `vue.config.js` 注入：

- `process.env.IS_ELECTRON = true`
- `process.env.IS_TAURI = true`
- `process.platform = "win32"`

这样前端会复用旧 Electron 桌面代码路径，再由 `tauriBridge.js` 转发到 Tauri IPC。

## 已知差异

- 只保证 Windows 端可用；macOS/Linux 专属能力不迁移。
- Last.fm 授权会走系统浏览器，旧 Electron 子窗口回调流程不可用。
- WebView2 媒体流量代理需要重启应用后生效；API 请求代理会随请求参数即时生效。
- 默认解灰源为 ytdl、bilibili、pyncm、kugou；ytdl 需要系统安装 `yt-dlp`。
- Tauri 生产窗口依赖本地 `127.0.0.1:27232` 服务。端口绑定失败时应用会中止启动，避免加载到其他本地进程页面。

## 发布前验证清单

Windows 版发布前至少验证：

- 安装包可安装、启动、卸载
- 扫码登录和登录态刷新正常
- 播放、暂停、切歌、进度拖动正常
- 歌词页、评论面板、评论发送/回复/点赞正常
- 音质切换后当前歌曲音源刷新正常
- 解灰可用，bilibili/ytdl 等来源失败时能回退
- 托盘菜单、关闭到托盘、全局快捷键正常
- 代理设置、缓存清理、`/player` 本地接口正常
- `cargo test` 通过

## 目录结构

```text
src/                    Vue 2 前端源码
src/api/                网易云 API 调用封装
src/components/         通用 UI 组件
src/electron/           旧 Electron 主进程/桌面能力实现
src/store/              Vuex 状态和本地持久化
src/utils/              播放器、请求、缓存、Tauri 桥接等工具
src/views/              页面视图
src-tauri/              Tauri 2 Windows 桌面端
public/                 静态资源
images/                 README 截图资源
```

## 许可和声明

本项目基于 YesPlayMusic 重构，仅供个人学习研究使用，禁止用于商业及非法用途。

原项目和 API 生态：

- YesPlayMusic: https://github.com/qier222/YesPlayMusic
- NeteaseCloudMusicApi: https://github.com/Binaryify/NeteaseCloudMusicApi
- UnblockNeteaseMusic: https://github.com/UnblockNeteaseMusic/server

本项目沿用 MIT License。

## 截图

![lyrics][lyrics-screenshot]
![library-dark][library-dark-screenshot]
![album][album-screenshot]
![home-2][home-2-screenshot]
![artist][artist-screenshot]
![search][search-screenshot]
![home][home-screenshot]
![explore][explore-screenshot]

[album-screenshot]: images/album.png
[artist-screenshot]: images/artist.png
[explore-screenshot]: images/explore.png
[home-screenshot]: images/home.png
[home-2-screenshot]: images/home-2.png
[lyrics-screenshot]: images/lyrics.png
[library-screenshot]: images/library.png
[library-dark-screenshot]: images/library-dark.png
[search-screenshot]: images/search.png
