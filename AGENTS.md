# AI 协作说明

本文件适用于本仓库。修改前先阅读 [Index.md](Index.md) 和相关源码；用户的当前要求优先。保留已有未提交修改，避免覆盖用户数据。

## 项目边界

Pi Workbench 2.x 的主要实现在 `tauri/`，通过本地服务和伪终端操作本机 Pi，不是远程终端服务。Electron 1.x 实现已退出当前源码树。Windows 已有本地验证；macOS 和 Linux 的构建与运行尚未验证。

## 修改约定

- 保持服务仅监听回环地址，并保留 API 令牌、敏感操作的来源校验与 WebSocket 校验；不要输出或提交 API Key、令牌、用户配置、终端会话内容。
- 工作台设置位于 `~/.pi-workbench-v2`，旧原型和 Electron 的读取回退不能覆盖旧数据；`PI_WORKBENCH_CONFIG_DIR` 可隔离配置。模型快捷项的并发写入需要文件锁及原子替换。
- 界面文案在 `tauri/assets/locales/zh-CN.json` 与 `en.json`，键和占位符须一致；Pi 自身及供应商输出不受工作台语言设置控制。
- 保持真实终端的输入、粘贴、输入法、滚动、链接、尺寸调整与会话清理；最小化不得结束会话。
- 不修改用户 Pi 扩展或 Clawd on Desk 集成，除非任务明确授权；不要通过禁用扩展来掩盖性能问题。
- 保持跨平台分支并区分实现与实机验证。对 Windows 代码签名和 Smart App Control 不作未经验证的保证。
- 用户操作、平台支持或功能变化时同步更新中英文 README。保留第三方许可证。

## 验证与交付

- 在 `tauri/` 运行 `npm ci`、`npm test`、`cargo test --release --bin pi-workbench` 和相应的 release 构建；按改动范围进行真实 Pi、桌面和平台验证。
- 发行前核对 EXE 的版本、资源、签名状态和启动情况；不要提交 `node_modules/`、`target/`、`tmp/`、本地工具链、日志、凭据或用户会话。
- 说明未验证的平台、测试限制和保留问题。提交与发布按用户当前授权执行。
