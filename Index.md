# 代码路径索引

本索引用于定位 Pi Workbench 的当前实现。用户安装、使用和平台支持说明见 [README.md](README.md) 和 [README.zh-CN.md](README.zh-CN.md)；AI 协作约定见 [AGENTS.md](AGENTS.md)。

## 目录与职责

| 路径 | 作用 |
| --- | --- |
| [desktop.js](desktop.js) | Electron 主进程：启动本地 Node 服务、创建桌面窗口、处理窗口和粘贴操作。 |
| [preload.js](preload.js) | 向页面暴露受限的桌面窗口操作桥接。 |
| [server.js](server.js) | 本地 HTTP 服务、静态资源及 API 路由、访问令牌校验、终端 WebSocket 和 `node-pty` 进程生命周期。 |
| [config.js](config.js) | 读取与保存设置、发现 Pi 模型、探测思考程度、识别供应商及查询额度、构造平台相关启动命令。 |
| [index.html](index.html) | 工作台页面骨架、桌面标题栏、侧栏、设置区、会话网格和最小化停靠栏。 |
| [client.js](client.js) | 页面交互：模型与额度、设置、语言和主题切换、终端会话、网格布局、拖动、重命名、最小化与确认弹窗。 |
| [style.css](style.css) | 页面布局、主题变量、终端容器、停靠栏、设置及弹窗样式。 |
| [locales/zh-CN.json](locales/zh-CN.json)、[locales/en.json](locales/en.json) | 简体中文与英文界面文案；键相同，占位符需保持一致。 |
| [test/config.test.js](test/config.test.js) | 模型清单解析、工作目录校验、跨平台命令和 Pi RPC 响应测试。 |
| [test/locales.test.js](test/locales.test.js) | 双语键和占位符一致性，以及旧设置的语言持久化测试。 |
| [package.json](package.json)、[package-lock.json](package-lock.json) | 依赖、启动与测试命令，以及 Windows x64 便携版打包配置和依赖锁定。 |
| [launch.ps1](launch.ps1) | 从源码目录使用本地 Electron 启动的 Windows 脚本。 |
| [workbench.ico](workbench.ico) | Windows 桌面窗口及发行程序图标。 |

## 常见修改入口

| 任务 | 从哪里开始 |
| --- | --- |
| 添加界面文案或语言 | `locales/`、`client.js` 的 `t`、`applyLanguage`、`initialize`，以及 `index.html` 的 `data-i18n`。添加语言时还需更新支持列表、资源路由和语言选择。 |
| 修改会话行为 | `client.js` 的 `createSession`、`arrange`、`updateDock`，以及 `server.js` 的 `/terminal` WebSocket 连接。 |
| 修改工作目录、主题或思考程度 | `index.html` 设置区、`client.js` 的 `saveSettings`，`server.js` 的 `/api/settings`，以及 `config.js` 的设置和校验逻辑。 |
| 接入供应商额度 | `config.js` 的 `quotaKind` 和 `quota`，`server.js` 的 `/api/quota`，`client.js` 的 `renderQuota`，以及对应的双语文案和 README。 |
| 修改桌面窗口或打包 | `desktop.js`、`preload.js`、`package.json`；打包文件默认输出到 `dist/`。 |

## 运行链路与本地数据

桌面模式由 `desktop.js` 启动 `server.js`，随后打开只绑定 `127.0.0.1` 的页面；页面调用 API 获取配置，并通过 WebSocket 连接 `node-pty` 启动 Pi。浏览器模式直接运行本地服务。页面刷新或关闭会结束该页面的终端连接，最小化会话则继续运行。

Pi 配置默认位于 `~/.pi/agent`，可由 `PI_CODING_AGENT_DIR` 指定；工作台设置默认位于 `~/.pi-workbench/settings.json`，可由 `PI_WORKBENCH_CONFIG_DIR` 指定。`PI_WORKBENCH_NODE` 可指定桌面模式使用的 Node 可执行文件。这些位置可能包含用户数据，排查时不要输出或提交凭据。
