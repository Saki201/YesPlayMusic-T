/**
 * Tauri 构建下替代 'electron' npm 包的空实现（见 vue.config.js 的 alias 配置）
 *
 * 渲染端存在裸 require('electron') 的代码（如 utils/nativeAlert.js），
 * web 打包时会解析到真实 electron 包并在模块加载时崩溃（其 index.js 依赖 fs）。
 * alias 到此空对象后，解构出的 dialog 等均为 undefined，相关代码自动走降级路径。
 */
module.exports = {};
