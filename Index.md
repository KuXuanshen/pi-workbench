# 代码路径索引

Pi Workbench 2.0 的主要实现位于 `tauri/`，使用 Rust 本地服务、系统 WebView 和 xterm.js。Electron 1.x 已退出当前源码树。

| 路径 | 职责 |
| --- | --- |
| [tauri/src/main.rs](tauri/src/main.rs) | 本地 HTTP/WebSocket 服务、令牌和来源校验、伪终端、桌面窗口及 API 路由。 |
| [tauri/src/config.rs](tauri/src/config.rs) | 设置与旧版回退、模型发现、思考程度、额度和模型快捷项持久化。 |
| [tauri/src/session_job.rs](tauri/src/session_job.rs) | Windows 会话进程组及关闭时的进程清理。 |
| [tauri/src/external_link.rs](tauri/src/external_link.rs) | HTTP(S) 链接校验和系统浏览器打开。 |
| [tauri/assets/index.html](tauri/assets/index.html) | 工作台页面骨架。 |
| [tauri/assets/client.js](tauri/assets/client.js) | 模型、会话、终端、布局、设置及多语言交互。 |
| [tauri/assets/ui-controls.js](tauri/assets/ui-controls.js) | 主题化图标和下拉控件。 |
| [tauri/assets/style.css](tauri/assets/style.css) | 页面布局、主题、终端和弹窗样式。 |
| [tauri/assets/locales](tauri/assets/locales) | 中文与英文界面文案。 |
| [tauri/tests](tauri/tests) | 控件、收藏、终端交互、链接及双语行为测试。 |
| [tauri/Cargo.toml](tauri/Cargo.toml)、[tauri/tauri.conf.json](tauri/tauri.conf.json) | Rust 依赖、版本和 Tauri 应用配置。 |
| [tauri/package.json](tauri/package.json) | 前端测试依赖与命令。 |

桌面入口创建只监听 `127.0.0.1` 的本地服务，页面以每次启动随机生成的令牌访问 API，并通过 WebSocket 连接真实伪终端。设置默认写入 `~/.pi-workbench-v2/`；旧 Tauri 原型和 Electron 设置仅作为缺失文件时的读取回退。目录和迁移行为详见 [README.zh-CN.md](README.zh-CN.md)。

当前 Windows 已有本地运行验证；macOS 和 Linux 仅有平台分支代码，尚未实机验证。旧目录、本机构建工具链、性能数据及临时验证副本不应提交到仓库。
