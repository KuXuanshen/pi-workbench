# Pi Workbench

**English** | [简体中文](README.zh-CN.md)

A local desktop workbench for running multiple [Pi](https://github.com/earendil-works/pi) coding sessions side by side. Pi Workbench discovers the providers and models available in your Pi configuration and lets you manage sessions in one window.

## Features

- **Multiple terminals:** create independent sessions, arrange them in a responsive grid, rename or reorder them, and minimize them to a dock without stopping them.
- **Models and usage:** discover models with `pi --list-models`; display MiniMax and Zhipu Coding Plan quotas and DeepSeek balances when available.
- **Thinking levels:** show the level resolved by Pi, choose a level for new sessions, or reset the workbench override.
- **Workspace settings:** choose an existing absolute directory for new sessions. Existing sessions keep their original directory.
- **Desktop and browser:** use the desktop application or a local browser, with themes, terminal scrolling, and text paste.

## Supported quota and balance providers

| Provider | What the workbench queries |
| --- | --- |
| MiniMax | Coding Plan quota, including the China and international endpoints |
| Zhipu AI / Z.AI | Coding Plan quota, including the China and international endpoints |
| DeepSeek | Available account balance |

The maintainer currently has access to these providers only, so quota integrations started with them. Contributions for other providers are welcome through issues or pull requests. Endpoint availability and credential requirements vary by provider; the international endpoints have not yet been verified with the maintainer's accounts.

## Requirements

| Requirement | Details |
| --- | --- |
| Operating system | Windows, macOS, or Linux. Tested on Windows; macOS and Linux still require hands-on verification. |
| Node.js | Version 22 or later, with npm. |
| Pi | Install Pi and ensure `pi --version` and `pi --list-models` work in the launch environment. |
| Authentication | Configure the providers and credentials you want to use in Pi. |

Native terminal dependencies may require platform-specific C/C++ build tools. Linux desktop mode requires a graphical session.

## Windows release

Download `Pi-Workbench-1.1.0-Windows-x64.exe` from [Releases](https://github.com/KuXuanshen/pi-workbench/releases), then run it. The portable executable does not install Node.js or Pi: both must already be available on `PATH`, and Pi must be configured with the providers you want to use. The executable is not code-signed, so Windows may display a publisher warning.

To build the Windows x64 portable executable from source on Windows:

```bash
npm install
npm run dist:win
```

The resulting executable is written to `dist/`. The desktop interface displays the application version.

## Getting started

On first launch, choose Simplified Chinese or English. You can change the interface language later in Settings; the choice is saved locally. The Pi terminal and provider output retain their own language settings.

Interface translations live in [`locales/zh-CN.json`](locales/zh-CN.json) and [`locales/en.json`](locales/en.json). When changing a message, update both files and preserve placeholders such as `{model}` and `{count}`.

```bash
git clone https://github.com/KuXuanshen/pi-workbench.git
cd pi-workbench
npm install
npm start
```

The desktop application looks for Node.js and Pi on `PATH`. If your graphical desktop does not inherit your shell environment, launch `npm start` from a terminal where both commands work. You can also set `PI_WORKBENCH_NODE` to the Node.js executable.

For a local browser instead:

```bash
npm run web
```

Alternatively, run `node server.js` and open the printed local address. The page requires the local server; opening the HTML file alone will not work. The address contains a temporary access token. Do not share it.

## Usage

1. Select the settings button in the workspace header and save an **existing absolute directory** for new sessions.
2. Expand a provider. The thinking selector shows Pi's resolved level for the model. Choose a different level if needed and select **＋** to create a session. Select **↺** to remove the workbench override and return to Pi's settings.
3. Drag the handle in a terminal header to reorder sessions. Select **−** to minimize a session and select its dock card to restore it.

Pi Workbench queries Pi's RPC state without loading extensions or sending prompts to determine the initial thinking level. Pi applies project and model settings and may limit a requested level to the model's supported levels. An extension loaded in an actual session may subsequently change that level. If the value cannot be determined, the selector reports it as unavailable instead of guessing.

Changing the directory or thinking level affects **new sessions only**. Closing the workbench or refreshing the page ends that page's terminal connections; minimizing a session does not.

## Configuration and security

- Pi's configuration directory follows `PI_CODING_AGENT_DIR`, defaulting to `~/.pi/agent`. Reopen or reload the workbench to discover updated models.
- Pi manages its own API keys and sign-in state. For supported quota providers, the workbench obtains the relevant key with `pi auth print-api-key` and sends it only to the recognized official quota endpoint. Keys are not returned to the page or saved in workbench settings. Other providers can still launch sessions, but quota lookup is unavailable. Some OAuth or other credential types may not provide an API key for quota lookup.
- The selected theme, directory, and model-level overrides are stored in `~/.pi-workbench/settings.json`. Set `PI_WORKBENCH_CONFIG_DIR` to use another settings directory. This file is not part of the repository.
- The server listens only on `127.0.0.1`; API and terminal connections require a random token. Pi and terminal commands run with the operating-system privileges of the user who started the workbench. Use it only in a trusted local environment.
- New Windows sessions set `PI_HARDWARE_CURSOR=1` to improve input-method positioning. An explicit Pi setting may override it.

## Testing

```bash
npm test
```

Automated tests cover model-list parsing, directory validation, platform-specific command construction, and RPC response validation. Desktop startup and real Pi sessions have been verified on Windows. macOS and Linux have not yet been tested on those systems.

## License

[MIT](LICENSE) © 2026 KuXuanshen
