//! 系统托盘（对照 src/electron/tray.js 的 YPMTrayWindowsImpl）
//!
//! - 左键：显示主窗口
//! - 右键菜单：播放/暂停（按状态二选一）、上/下一首、循环、喜欢/取消喜欢、退出
//! - tooltip / 播放状态 / 喜欢状态 / 图标主题由前端通过 updateTray* channel 推送

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::{
    image::Image,
    menu::{IconMenuItem, Menu, MenuEvent},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

use crate::settings::Settings;

static TRAY: Mutex<Option<TrayIcon>> = Mutex::new(None);
static IS_PLAYING: AtomicBool = AtomicBool::new(false);
static IS_LIKED: AtomicBool = AtomicBool::new(false);

fn icon(bytes: &'static [u8]) -> Option<Image<'static>> {
    Image::from_bytes(bytes).ok()
}

/// 按当前播放/喜欢状态构建托盘菜单
fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let playing = IS_PLAYING.load(Ordering::Relaxed);
    let liked = IS_LIKED.load(Ordering::Relaxed);

    let play_pause = if playing {
        IconMenuItem::with_id(
            app,
            "play",
            "暂停",
            true,
            icon(include_bytes!("../../public/img/icons/pause.png")),
            None::<&str>,
        )?
    } else {
        IconMenuItem::with_id(
            app,
            "play",
            "播放",
            true,
            icon(include_bytes!("../../public/img/icons/play.png")),
            None::<&str>,
        )?
    };

    let previous = IconMenuItem::with_id(
        app,
        "previous",
        "上一首",
        true,
        icon(include_bytes!("../../public/img/icons/left.png")),
        None::<&str>,
    )?;
    let next = IconMenuItem::with_id(
        app,
        "next",
        "下一首",
        true,
        icon(include_bytes!("../../public/img/icons/right.png")),
        None::<&str>,
    )?;
    let repeat = IconMenuItem::with_id(
        app,
        "repeat",
        "循环播放",
        true,
        icon(include_bytes!("../../public/img/icons/repeat.png")),
        None::<&str>,
    )?;
    let like = if liked {
        IconMenuItem::with_id(
            app,
            "like",
            "取消喜欢",
            true,
            icon(include_bytes!("../../public/img/icons/unlike.png")),
            None::<&str>,
        )?
    } else {
        IconMenuItem::with_id(
            app,
            "like",
            "加入喜欢",
            true,
            icon(include_bytes!("../../public/img/icons/like.png")),
            None::<&str>,
        )?
    };
    let exit = IconMenuItem::with_id(
        app,
        "exit",
        "退出",
        true,
        icon(include_bytes!("../../public/img/icons/exit.png")),
        None::<&str>,
    )?;

    Menu::with_items(app, &[&play_pause, &previous, &next, &repeat, &like, &exit])
}

/// 根据主题取托盘图标（对照 tray.js updateIcon：auto 时跟随系统暗色 → light 图标）
fn tray_image(app: &AppHandle) -> Option<Image<'static>> {
    let setting = app
        .state::<Settings>()
        .get_setting_str("trayIconTheme")
        .unwrap_or_else(|| "auto".into());

    let theme = match setting.as_str() {
        "light" | "dark" => setting,
        _ => {
            // auto：系统暗色用 light 图标，反之 dark
            let dark = app
                .get_webview_window("main")
                .and_then(|w| w.theme().ok())
                .map(|t| t == tauri::Theme::Dark)
                .unwrap_or(false);
            if dark { "light".into() } else { "dark".into() }
        }
    };

    if theme == "light" {
        icon(include_bytes!("../../public/img/icons/menu-light@88.png"))
    } else {
        icon(include_bytes!("../../public/img/icons/menu-dark@88.png"))
    }
}

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    let emit = |channel: &str| {
        if let Some(win) = app.get_webview_window("main") {
            let _ = win.emit(channel, serde_json::json!([]));
        }
    };
    match event.id().as_ref() {
        "play" => emit("play"),
        "previous" => emit("previous"),
        "next" => emit("next"),
        "repeat" => emit("repeat"),
        "like" => emit("like"),
        "exit" => app.exit(0),
        _ => {}
    }
}

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_menu(app)?;

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("YesPlayMusic-T")
        .menu(&menu)
        // 左键不弹菜单（弹菜单走右键），左键单独处理为显示窗口
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| on_menu_event(app, event))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.unminimize();
                    let _ = win.set_focus();
                }
            }
        });

    if let Some(img) = tray_image(app) {
        builder = builder.icon(img);
    }

    let tray = builder.build(app)?;
    *TRAY.lock().unwrap() = Some(tray);
    Ok(())
}

pub fn set_tooltip(title: &str) {
    if let Some(tray) = TRAY.lock().unwrap().as_ref() {
        let _ = tray.set_tooltip(Some(title));
    }
}

pub fn set_playing(app: &AppHandle, playing: bool) {
    IS_PLAYING.store(playing, Ordering::Relaxed);
    rebuild_menu(app);
}

pub fn set_liked(app: &AppHandle, liked: bool) {
    IS_LIKED.store(liked, Ordering::Relaxed);
    rebuild_menu(app);
}

pub fn update_icon(app: &AppHandle) {
    if let Some(tray) = TRAY.lock().unwrap().as_ref() {
        if let Some(img) = tray_image(app) {
            let _ = tray.set_icon(Some(img));
        }
    }
}

fn rebuild_menu(app: &AppHandle) {
    if let Some(tray) = TRAY.lock().unwrap().as_ref() {
        if let Ok(menu) = build_menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}
