# Pi Workbench

[English](README.md) | **简体中文**

Pi Workbench 是一个本地多会话工作台，用来在同一窗口运行多个 [Pi](https://github.com/earendil-works/pi) 编程会话。它从本机 Pi 配置发现可用供应商和模型，支持查看部分供应商的额度、为模型设置默认思考程度，并管理并行终端。

## 功能

- **多会话**：按模型创建独立的 Pi 终端，自动网格排列；可重命名、拖动调整顺序或最小化到底部停靠栏。最小化后会话继续运行。
- **模型与额度**：读取 `pi --list-models`，列出 Pi 当前可用的供应商和模型；支持查询 MiniMax、智谱 Coding Plan 的额度以及 DeepSeek 账户余额。
- **会话设置**：在页面设置新会话的工作目录，并为每个支持思考能力的模型选择默认思考程度。
- **桌面体验**：提供桌面窗口、四种主题、终端滚轮和文本粘贴；也可在本机浏览器中使用。

## 支持查询额度和余额的供应商

| 供应商 | 工作台查询的内容 |
| --- | --- |
| MiniMax | Coding Plan 额度，包含国内和国际接口 |
| 智谱 AI / Z.AI | Coding Plan 额度，包含国内和国际接口 |
| DeepSeek | 账户可用余额 |

目前只接入这些供应商，是因为本人只有这几种来源。欢迎大家通过 Issue 或 Pull Request 补充其他供应商。接口可用性和凭据要求因供应商而异；国际接口尚未通过本人的账号实测。

## 系统要求

| 要求 | 说明 |
| --- | --- |
| 系统 | Windows、macOS 或 Linux；目前仅在 Windows 实机验证 |
| Node.js | 22 或更高版本，包含 npm |
| Pi | 已安装 Pi，且 `pi --version` 与 `pi --list-models` 能在启动工作台的环境中运行 |
| Pi 认证 | 已通过 Pi 配置所需供应商和认证信息 |

安装原生终端依赖可能需要对应系统的 C/C++ 编译工具；Linux 桌面窗口需要可用的图形会话。

## Windows 发行版

从 [Releases](https://github.com/KuXuanshen/pi-workbench/releases) 下载并运行 `Pi-Workbench-1.1.0-Windows-x64.exe`。该便携程序不会安装 Node.js 或 Pi：运行前仍需在本机安装两者、确保命令在 `PATH` 中可用，并在 Pi 中配置所需供应商。程序未进行代码签名，Windows 可能显示发布者提示。

在 Windows 上从源码构建 x64 便携程序：

```bash
npm install
npm run dist:win
```

生成的可执行文件位于 `dist/`；桌面界面会显示应用版本。

## 安装和启动

首次启动时选择简体中文或英文，之后可在设置中切换；选择会保存在本机。Pi 终端及供应商返回内容使用其自身的语言设置。

界面翻译分别保存在 [`locales/zh-CN.json`](locales/zh-CN.json) 和 [`locales/en.json`](locales/en.json)。修改文案时请同步更新两个文件，并保留 `{model}`、`{count}` 等占位符。

```bash
git clone https://github.com/KuXuanshen/pi-workbench.git
cd pi-workbench
npm install
npm start
```

桌面窗口默认从 `PATH` 查找 Node.js 和 Pi。如果图形桌面没有继承你的 shell 环境，可从配置好环境的终端执行 `npm start`；也可通过 `PI_WORKBENCH_NODE` 指定 Node.js 可执行文件。

只使用浏览器时：

```bash
npm run web
```

也可以执行 `node server.js`，手动打开终端输出的本机地址。页面需要本地服务，不能只打开 HTML 文件。每次启动生成的地址包含临时访问令牌，请勿分享。

## 设置和日常使用

1. 点击工作区右上角的设置按钮，填写**已存在的绝对目录**并保存。此目录只影响之后创建的会话。
2. 在左侧展开供应商，模型旁直接显示 Pi 解析出的思考程度；需要时可另选程度，再点击 **＋** 创建会话。点击 **↺** 可移除工作台的覆盖设置，恢复 Pi 自身设置。
3. 拖动终端标题栏左侧的手柄调整顺序；点击 **−** 收起到停靠栏，点击停靠卡片恢复。

关闭工作台或刷新页面会结束该页面的终端连接。最小化子窗口不会结束会话。

工作台通过 Pi 的 RPC 状态读取初始思考程度，不加载扩展，也不发送提示词。Pi 会结合项目和模型配置，并可能将请求的程度调整到模型支持的范围。实际会话中加载的扩展可能随后改变该值。如果无法读取，控件会显示无法确定，不会猜测。

## 配置、认证和安全

- Pi 配置目录遵循 `PI_CODING_AGENT_DIR`；未设置时使用 `~/.pi/agent`。重新打开工作台或刷新页面会重新读取可用模型。
- Pi 自己管理 API Key 和登录状态。工作台使用 `pi auth print-api-key` 获取支持的供应商的密钥，并仅将其发送到对应的官方额度接口；密钥不会返回给页面或写入工作台配置。其他供应商仍可启动会话，但会显示额度查询暂不支持。某些 OAuth 或其他认证方式可能不能提供额度查询所需的 API Key。
- 主题、目录和模型默认思考程度存放在 `~/.pi-workbench/settings.json`；可通过 `PI_WORKBENCH_CONFIG_DIR` 指定配置目录。该文件不属于项目源代码。
- 本地服务只监听 `127.0.0.1`，接口和终端连接受随机令牌保护。Pi 和终端使用启动工作台的用户权限，请只在可信的本机环境中使用。
- Windows 新会话会设置 `PI_HARDWARE_CURSOR=1` 以改善输入法定位；Pi 自身的显式设置可能覆盖该环境变量。

## 测试

```bash
npm test
```

测试覆盖模型清单解析、工作目录校验、各平台的命令参数构造和 RPC 响应校验。Windows 上已验证桌面启动、实际 Pi 会话、目录和思考程度设置；macOS 和 Linux 仍需在对应系统上实机验收。
