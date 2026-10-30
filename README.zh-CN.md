# Pi Workbench

[English](README.md) | **简体中文**

Pi Workbench 是本地多会话工作台，可在同一窗口并排运行多个 [Pi](https://github.com/earendil-works/pi) 编程终端。2.0 将桌面框架和本地服务迁移到 Tauri 与 Rust，Pi 仍作为本机真实终端进程运行。

## 功能

- 创建独立 Pi 会话并自适应网格排列；支持拖动排序、重命名、最小化和恢复，最小化不会结束进程。
- 发现 Pi 已配置的供应商和模型，支持模型收藏、最近使用及模型默认思考程度。
- 查询已支持供应商的额度，显示刷新状态，并按需检查本机运行环境。
- 设置新会话工作目录、主题和中英文界面；终端中的 HTTP(S) 链接通过系统浏览器打开。
- 本地服务仅监听回环地址，每次启动使用独立访问令牌。Pi 凭据保留在 Pi 配置中，请勿在问题报告中分享。

工作台不保存终端布局，也不替代 Pi 自带的会话历史功能。2.0 桌面版不提供浏览器独立运行模式。

## 支持查询额度和余额的供应商

| 供应商 | 查询内容 |
| --- | --- |
| MiniMax | Coding Plan 额度，包括国内和国际接口 |
| 智谱 AI / Z.AI | Coding Plan 额度，包括国内和国际接口 |
| DeepSeek | 账户可用余额 |

目前只接入这些供应商，是因为本人只有这几种来源。欢迎通过 Issue 或 Pull Request 补充其他供应商。国际接口尚未使用本人的账号验证；具体可用性取决于供应商接口和凭据。

## 系统要求与平台状态

- Windows x64，安装 Microsoft Edge WebView2 Runtime、Node.js 22.19 或更高版本和 Pi，并确保启动环境的 `PATH` 能找到 Pi；请先在 Pi 中配置模型和凭据。
- Windows 版本已在本机测试。Tauri 本身支持 macOS 和 Linux，但本实现**尚未在这两个平台构建或实测**，终端、剪贴板、窗口和进程清理等行为仍需验证。
- 2.0 使用系统 WebView，Windows 可执行文件不包含 Pi、Node.js 或 WebView2。

## 下载与使用

从 [Releases](https://github.com/KuXuanshen/pi-workbench/releases) 下载并运行 `Pi-Workbench-2.0.0-Windows-x64.exe`。它是可直接运行的便携可执行文件，不是安装程序。首次启动时选择界面语言，以后可在设置中更改。

该程序未签名，Windows SmartScreen 或智能应用控制可能拦截；构建成功或发布者提示不能保证程序在所有设备上可运行。

## 从源码构建与测试

安装 Rust、对应平台的 Tauri 构建依赖、Node.js 和 Pi。Windows 构建需要 MSVC C++ 工具链和 Windows SDK。

```bash
git clone https://github.com/KuXuanshen/pi-workbench.git
cd pi-workbench/tauri
npm ci
npm test
cargo test --release --bin pi-workbench
cargo build --release --bin pi-workbench
```

Windows 构建产物位于 `tauri/target/release/pi-workbench.exe`。其他系统需要在相应平台构建并验证。

## 设置与隐私

2.0 将设置和模型快捷项保存在 `~/.pi-workbench-v2/`。若相应的 v2 文件不存在，会先读取旧 Tauri 原型的 `~/.pi-workbench-tauri-prototype/`；设置文件还会回退读取 Electron 1.x 的 `~/.pi-workbench/settings.json`。下次修改时写入 v2 目录，不覆盖旧文件。设置 `PI_WORKBENCH_CONFIG_DIR` 可指定目录，此时不读取旧目录。

Pi 配置默认位于 `~/.pi/agent`，可使用 Pi 的 `PI_CODING_AGENT_DIR` 覆盖。请勿提交凭据、会话内容、本机配置或构建产物，也不要在报告中粘贴这些信息。

## 项目路径

代码地图见 [`Index.md`](Index.md)，协作约定见 [`AGENTS.md`](AGENTS.md)。界面文案位于 [`tauri/assets/locales`](tauri/assets/locales)，修改时需同步中英文和占位符。第三方终端组件的许可证保留在 [`tauri/assets`](tauri/assets)。

## 许可证

[MIT](LICENSE)。第三方依赖适用各自的许可证。
