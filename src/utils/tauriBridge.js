/**
 * Tauri 桥接层（仅 Windows 桌面版）
 *
 * 在 Tauri 环境下模拟 Electron 渲染进程的 `window.require('electron')` 接口，
 * 将散落在前端的 ipcRenderer 调用全部转发到 Tauri IPC，
 * 使现有 Electron 耦合代码（Player.js / settings.vue / Win32Titlebar.vue 等）零改动运行。
 *
 * 同时承担 Electron 应用菜单 accelerator 的职责（menu.js 在 Tauri 下不存在），
 * 通过 keydown 监听实现应用内快捷键。
 *
 * 必须作为 main.js 的第一个 import（早于 store / App.vue 等模块级
 * window.require 调用执行）。
 */

import defaultShortcuts from '@/utils/shortcuts';

if (process.env.IS_TAURI) {
  const tauri = () => window.__TAURI__;

  // channel -> [callback]，应用内快捷键直接走本地分发
  const localListeners = {};

  const dispatchLocal = channel => {
    (localListeners[channel] || []).forEach(cb => cb({}));
  };

  const ipcRenderer = {
    /** 渲染进程 → 主进程，单向消息（对应 Electron ipcRenderer.send） */
    send(channel, ...args) {
      tauri()
        ?.core.invoke('ipc_send', { channel, args })
        .catch(err => console.error(`[tauriBridge] send(${channel})`, err));
    },

    /** 渲染进程 → 主进程，带返回值（对应 Electron ipcRenderer.invoke） */
    invoke(channel, ...args) {
      if (channel === 'unblock-music') {
        return tauri().core.invoke('unblock_music', {
          sourceListString: args[0] ?? null,
          ncmTrack: args[1],
          context: args[2] ?? {},
        });
      }
      return tauri().core.invoke('ipc_send', { channel, args });
    },

    /** 主进程 → 渲染进程事件监听（对应 Electron ipcRenderer.on）
     *  约定：Rust 侧统一以 JSON 数组作为 payload，这里展开为多参数回调 */
    on(channel, callback) {
      (localListeners[channel] = localListeners[channel] || []).push(callback);
      tauri()?.event.listen(channel, event => {
        const args = Array.isArray(event.payload)
          ? event.payload
          : event.payload === null || event.payload === undefined
          ? []
          : [event.payload];
        callback({}, ...args);
      });
    },
  };

  window.require = mod => {
    if (mod === 'electron') return { ipcRenderer };
    if (mod === 'os') return { platform: () => 'win32' };
    console.warn(`[tauriBridge] window.require('${mod}') 未实现，返回空对象`);
    return {};
  };

  // 外部链接交给系统浏览器打开（对照 Electron new-window → shell.openExternal）
  const originalOpen = window.open.bind(window);
  window.open = (url, ...rest) => {
    if (
      typeof url === 'string' &&
      /^https?:\/\//i.test(url) &&
      !url.startsWith(window.location.origin)
    ) {
      ipcRenderer.send('openExternal', url);
      return null;
    }
    return originalOpen(url, ...rest);
  };

  // ---- 应用内快捷键（对照 src/electron/menu.js 的 accelerator）----

  /** 判断 keydown 事件是否命中 Electron accelerator（如 'CommandOrControl+P'） */
  const matchAccelerator = (e, accel) => {
    if (!accel) return false;
    const parts = accel.split('+');
    const key = parts.pop();
    const mods = parts.map(p => p.toLowerCase());
    const wantCtrl =
      mods.includes('commandorcontrol') ||
      mods.includes('cmdorctrl') ||
      mods.includes('control') ||
      mods.includes('ctrl');
    const wantAlt = mods.includes('alt') || mods.includes('option');
    const wantShift = mods.includes('shift');
    if (
      e.ctrlKey !== wantCtrl ||
      e.altKey !== wantAlt ||
      e.shiftKey !== wantShift
    ) {
      return false;
    }
    const arrowMap = {
      up: 'arrowup',
      down: 'arrowdown',
      left: 'arrowleft',
      right: 'arrowright',
    };
    const k = key.toLowerCase();
    return e.key.toLowerCase() === (arrowMap[k] || k);
  };

  const getShortcuts = () => {
    try {
      const settings = JSON.parse(localStorage.getItem('settings')) || {};
      return Array.isArray(settings.shortcuts) && settings.shortcuts.length
        ? settings.shortcuts
        : defaultShortcuts;
    } catch (e) {
      return defaultShortcuts;
    }
  };

  window.addEventListener('keydown', e => {
    // 固定快捷键（menu.js 中硬编码的 accelerator）
    if (matchAccelerator(e, 'CmdOrCtrl+F')) {
      e.preventDefault();
      return dispatchLocal('search');
    }
    if (matchAccelerator(e, 'Alt+R')) {
      e.preventDefault();
      return dispatchLocal('repeat');
    }
    if (matchAccelerator(e, 'Alt+S')) {
      e.preventDefault();
      return dispatchLocal('shuffle');
    }

    // 可配置快捷键（settings.shortcuts 的 shortcut 字段，对应 Controls 菜单）
    for (const s of getShortcuts()) {
      if (s.id === 'minimize') continue; // Electron 应用内未注册该项
      if (matchAccelerator(e, s.shortcut)) {
        e.preventDefault();
        return dispatchLocal(s.id);
      }
    }
  });

  // 每秒向 Rust 端推送播放状态，供 127.0.0.1:27232/player 对外接口使用
  // （对应 Electron 版 background.js 通过 executeJavaScript 拉取的逻辑）
  setInterval(() => {
    const player = window.yesplaymusic?.player;
    if (!player) return;
    try {
      tauri()
        ?.core.invoke('update_player_state', {
          state: {
            currentTrack: player._isPersonalFM
              ? player._personalFMTrack
              : player._currentTrack,
            progress: player._progress,
          },
        })
        .catch(() => {});
    } catch (e) {
      /* 播放器未就绪时静默忽略 */
    }
  }, 1000);
}
