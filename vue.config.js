const webpack = require('webpack');
const path = require('path');
function resolve(dir) {
  return path.join(__dirname, dir);
}

module.exports = {
  // 生产环境打包不输出 map
  productionSourceMap: false,
  devServer: {
    disableHostCheck: true,
    port: process.env.DEV_SERVER_PORT || 8080,
    proxy: {
      // Tauri 开发模式：代理到 Rust 侧 axum 服务（路由本身就挂在 /api 下，不重写路径）
      // 其他模式：代理到本地 Node 版 NeteaseCloudMusicApi
      '^/api': process.env.TAURI_BUILD
        ? {
            target: 'http://127.0.0.1:27232',
            changeOrigin: true,
          }
        : {
            target: 'http://localhost:3000',
            changeOrigin: true,
            pathRewrite: {
              '^/api': '/',
            },
          },
    },
  },
  pwa: {
    name: 'YesPlayMusic-T',
    iconPaths: {
      favicon32: 'img/icons/favicon-32x32.png',
    },
    themeColor: '#ffffff00',
    manifestOptions: {
      background_color: '#335eea',
    },
    // workboxOptions: {
    //   swSrc: "dev/sw.js",
    // },
  },
  pages: {
    index: {
      entry: 'src/main.js',
      template: 'public/index.html',
      filename: 'index.html',
      title: 'YesPlayMusic-T',
      chunks: ['main', 'chunk-vendors', 'chunk-common', 'index'],
    },
  },
  chainWebpack(config) {
    // Tauri 构建：复用 Electron 渲染端的编译期常量，让前端走桌面端代码路径
    // process.platform 固定为 win32（仅出 Windows 版），保证 utils/platform.js 判断正确
    if (process.env.TAURI_BUILD) {
      config.plugin('define').tap(args => {
        args[0]['process.env'].IS_ELECTRON = true;
        args[0]['process.env'].IS_TAURI = true;
        args[0]['process.platform'] = JSON.stringify('win32');
        return args;
      });
      // 裸 require('electron') 的渲染端代码（如 nativeAlert）在 web 打包下会
      // 解析到真实 electron 包并在加载时崩溃，alias 到空实现走降级路径
      config.resolve.alias.set(
        'electron',
        resolve('src/utils/electronStub.js')
      );
    }
    config.module.rules.delete('svg');
    config.module.rule('svg').exclude.add(resolve('src/assets/icons')).end();
    config.module
      .rule('icons')
      .test(/\.svg$/)
      .include.add(resolve('src/assets/icons'))
      .end()
      .use('svg-sprite-loader')
      .loader('svg-sprite-loader')
      .options({
        symbolId: 'icon-[name]',
      })
      .end();
    config.module
      .rule('napi')
      .test(/\.node$/)
      .use('node-loader')
      .loader('node-loader')
      .end();

    config.module
      .rule('webpack4_es_fallback')
      .test(/\.js$/)
      .include.add(/node_modules/)
      .end()
      .use('esbuild-loader')
      .loader('esbuild-loader')
      .options({ target: 'es2015', format: 'cjs' })
      .end();

    // LimitChunkCountPlugin 可以通过合并块来对块进行后期处理。用以解决 chunk 包太多的问题
    config.plugin('chunkPlugin').use(webpack.optimize.LimitChunkCountPlugin, [
      {
        maxChunks: 3,
        minChunkSize: 10_000,
      },
    ]);
  },
  // 添加插件的配置
  pluginOptions: {
    // electron-builder的配置文件
    electronBuilder: {
      nodeIntegration: true,
      externals: ['@unblockneteasemusic/rust-napi'],
      builderOptions: {
        productName: 'YesPlayMusic-T',
        copyright: 'Copyright © YesPlayMusic-T',
        // compression: "maximum", // 机器好的可以打开，配置压缩，开启后会让 .AppImage 格式的客户端启动缓慢
        asar: true,
        publish: [
          {
            provider: 'github',
            owner: 'Saki201',
            repo: 'YesPlayMusic-T',
            vPrefixedTagName: true,
            releaseType: 'draft',
          },
        ],
        directories: {
          output: 'dist_electron',
        },
        mac: {
          target: [
            {
              target: 'dmg',
              arch: ['x64', 'arm64', 'universal'],
            },
          ],
          artifactName: '${productName}-${os}-${version}-${arch}.${ext}',
          category: 'public.app-category.music',
          darkModeSupport: true,
        },
        win: {
          target: [
            {
              target: 'portable',
              arch: ['x64'],
            },
            {
              target: 'nsis',
              arch: ['x64'],
            },
          ],
          publisherName: 'YesPlayMusic-T',
          icon: 'build/icons/icon.ico',
          publish: ['github'],
        },
        linux: {
          target: [
            {
              target: 'AppImage',
              arch: ['x64'],
            },
            {
              target: 'tar.gz',
              arch: ['x64', 'arm64'],
            },
            {
              target: 'deb',
              arch: ['x64', 'armv7l', 'arm64'],
            },
            {
              target: 'rpm',
              arch: ['x64'],
            },
            {
              target: 'snap',
              arch: ['x64'],
            },
            {
              target: 'pacman',
              arch: ['x64'],
            },
          ],
          category: 'Music',
          icon: './build/icon.icns',
        },
        dmg: {
          icon: 'build/icons/icon.icns',
        },
        nsis: {
          oneClick: true,
          perMachine: true,
          deleteAppDataOnUninstall: true,
        },
      },
      // 主线程的配置文件
      chainWebpackMainProcess: config => {
        config.plugin('define').tap(args => {
          args[0]['IS_ELECTRON'] = true;
          return args;
        });
        config.resolve.alias.set(
          'jsbi',
          path.join(__dirname, 'node_modules/jsbi/dist/jsbi-cjs.js')
        );

        config.module
          .rule('webpack4_es_fallback')
          .test(/\.js$/)
          .include.add(/node_modules/)
          .end()
          .use('esbuild-loader')
          .loader('esbuild-loader')
          .options({ target: 'es2015', format: 'cjs' })
          .end();
      },
      // 渲染线程的配置文件
      chainWebpackRendererProcess: config => {
        // 渲染线程的一些其他配置
        // Chain webpack config for electron renderer process only
        // The following example will set IS_ELECTRON to true in your app
        config.plugin('define').tap(args => {
          args[0]['IS_ELECTRON'] = true;
          return args;
        });
      },
      // 主入口文件
      // mainProcessFile: 'src/main.js',
      // mainProcessArgs: []
    },
  },
};
