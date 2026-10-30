# Pi Workbench

**English** | [简体中文](README.zh-CN.md)

A local desktop workbench for running multiple [Pi](https://github.com/earendil-works/pi) coding sessions side by side. Version 2 replaces the Electron desktop shell with Tauri and Rust while keeping Pi as a real local terminal process.

## Features

- Create independent Pi sessions in a responsive grid; reorder, rename, minimize, and restore them without stopping the process.
- Discover configured Pi providers and models; pin favorites, access recent models, and set the default thinking level for each model.
- View supported provider quotas, refresh status, and run an on-demand environment check.
- Choose the working directory for new sessions, a theme, and English or Simplified Chinese. Terminal HTTP(S) links open in the system browser.
- The desktop service listens on the local loopback interface and requires a per-launch access token. Pi credentials stay in the Pi configuration; do not share them in issue reports.

The workbench does not restore terminal layouts or replace Pi's own session history. The 2.0 application is desktop-only and does not offer a standalone browser mode.

## Supported quota and balance providers

| Provider | Information queried |
| --- | --- |
| MiniMax | Coding Plan quota, China and international endpoints |
| Zhipu AI / Z.AI | Coding Plan quota, China and international endpoints |
| DeepSeek | Account balance |

The maintainer has access only to these sources, so integrations for other providers are welcome through issues or pull requests. International endpoints have not been verified with the maintainer's accounts; support depends on the provider's API and credentials.

## Requirements and platform status

- Windows x64 with Microsoft Edge WebView2 Runtime, Node.js 22.19 or later, and Pi installed and available on `PATH`. Configure models and credentials in Pi before launching.
- The Windows build has been tested locally. Tauri also supports macOS and Linux, but this implementation has **not** been built or tested on those platforms; dependencies, terminal behavior, clipboard, window handling and process cleanup require verification.
- Pi Workbench 2.0 uses the system WebView. The Windows executable does not bundle Pi, Node.js or WebView2.

## Download and run

Download the Windows x64 executable from [Releases](https://github.com/KuXuanshen/pi-workbench/releases). Run `Pi-Workbench-2.0.0-Windows-x64.exe`; it is a portable executable, not an installer. The first launch offers a language choice, which can later be changed in Settings.

The executable is unsigned. Windows SmartScreen or Smart App Control may block it; a successful build or a publisher warning does not establish trust or guarantee execution.

## Build and test

Install the Rust toolchain, the platform's Tauri build prerequisites, Node.js and Pi. Windows builds require the MSVC C++ toolchain and Windows SDK.

```bash
git clone https://github.com/KuXuanshen/pi-workbench.git
cd pi-workbench/tauri
npm ci
npm test
cargo test --release --bin pi-workbench
cargo build --release --bin pi-workbench
```

On Windows the executable is `tauri/target/release/pi-workbench.exe`. A build for another operating system must be produced and tested on that platform.

## Settings and privacy

Version 2 stores settings and model shortcuts under `~/.pi-workbench-v2/`. If a corresponding v2 file does not yet exist, it reads the previous Tauri prototype's `~/.pi-workbench-tauri-prototype/` data; settings additionally fall back to the Electron 1.x `~/.pi-workbench/settings.json`. On the next change, the relevant file is written to the v2 directory without overwriting the old data. `PI_WORKBENCH_CONFIG_DIR` overrides the v2 directory and disables these fallbacks.

Pi's own configuration defaults to `~/.pi/agent`; Pi supports `PI_CODING_AGENT_DIR` to override it. Credentials, session content, local build outputs and personal settings should never be committed or included in reports.

## Project paths

See [`Index.md`](Index.md) for the code map and [`AGENTS.md`](AGENTS.md) for contributor and AI instructions. UI messages are in [`tauri/assets/locales`](tauri/assets/locales); update both languages and preserve placeholders. Third-party terminal components retain their license notices in [`tauri/assets`](tauri/assets).

## License

[MIT](LICENSE). Third-party dependencies have their own licenses.
