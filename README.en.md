<div align="center">

# CJK AutoCorrect Desktop

**A local desktop formatter for CJK text**

[Simplified Chinese](./README.md) | [English](./README.en.md)

[![Tauri](https://img.shields.io/badge/Tauri-v2-blue?logo=tauri)](https://v2.tauri.app/)
[![React](https://img.shields.io/badge/React-19-61dafb?logo=react)](https://react.dev/)
[![Rust](https://img.shields.io/badge/Rust-2021-000000?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

</div>

---

CJK AutoCorrect Desktop is a local desktop client powered by the [autocorrect](https://github.com/huacnlee/autocorrect) engine. It formats Chinese, Japanese, and Korean text by adding spacing between CJK characters and Latin letters or numbers, normalizing punctuation, and letting you choose between standard and strict formatting modes.

## Features

- **Local instant formatting** - Paste or type text, format it in one click, then copy or clear the result. `Cmd/Ctrl + Enter` also formats the current input.
- **Standard / strict modes** - Standard mode handles common CJK typography fixes. Strict mode invokes `autocorrect --strict` for stricter rules.
- **Default mode sync** - Choose the default formatting mode in Settings, and the main formatter view automatically follows that preference.
- **Clipboard workflow** - Read text from the clipboard and write formatted output back to the clipboard.
- **Global shortcut** - The default shortcut is `Cmd/Ctrl + Shift + F`, and custom shortcuts can be recorded from the Settings page.
- **Enhanced history** - Only changed formatting results are saved. History supports search, mode filtering, result copy, restore to the main editor, detail view, and full clear.
- **Engine detection and custom path** - The app auto-detects the local `autocorrect` CLI, with an optional manual binary path.
- **System integration** - System tray support, close-to-tray behavior, and optional launch at login.
- **Appearance settings** - Light, dark, and system themes.

## Screenshot

![CJK AutoCorrect Desktop main window](./docs/images/main-window.png)

## Install and Use

CJK AutoCorrect Desktop uses the local `autocorrect` CLI as its formatting engine.

1. Download the installer for your platform from [Releases](../../releases).
2. Install the local `autocorrect` CLI.
3. Open the app and make sure the bottom status bar shows that `autocorrect` is available.

You can open the app and Settings without `autocorrect`, but formatting requires the CLI to be installed. After installing it, restart the app or set the `autocorrect` binary path manually in Settings.

### Install autocorrect

```bash
# macOS
brew install huacnlee/tap/autocorrect

# Windows
scoop install autocorrect

# Or with Cargo
cargo install autocorrect
```

## Development

### Development Requirements

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/) >= 8
- [Rust](https://www.rust-lang.org/tools/install) >= 1.77
- [autocorrect](https://github.com/huacnlee/autocorrect) >= 2.0, for local formatting tests

### Start the Dev App

```bash
# Clone the repository
git clone https://github.com/cuitz/cjk-autocorrect-desktop.git
cd cjk-autocorrect-desktop

# Install frontend dependencies
pnpm install

# Start the Tauri development app
pnpm tauri dev
```

### Build Locally

```bash
pnpm tauri build
```

Build artifacts are generated under `src-tauri/target/release/bundle/`.

## Architecture

```
┌─────────────────────────────────────────────┐
│                 Frontend                     │
│   React 19 · TypeScript · Tailwind CSS 4    │
│              Zustand · Vite 7                │
├─────────────────────────────────────────────┤
│               Tauri v2 Bridge               │
├─────────────────────────────────────────────┤
│                 Backend                      │
│          Rust · FormatterEngine              │
│        autocorrect CLI (stdin pipe)          │
└─────────────────────────────────────────────┘
```

| Layer | Technology | Description |
|------|------------|-------------|
| Frontend | React 19 + TypeScript | Component-based UI and Zustand state management |
| Styling | Tailwind CSS 4 | CSS variable driven Stone/Indigo design tokens |
| Bridge | Tauri v2 Commands | Typed frontend/backend communication |
| Backend | Rust | Formatting service, configuration, history storage, and app integration |
| Engine | autocorrect CLI | Invoked through stdin/stdout |

### Project Structure

```
src/                          # Frontend source
├── components/
│   ├── FormatPage.tsx        # Main formatter view
│   ├── HistoryPage.tsx       # History view
│   └── SettingsPage.tsx      # Settings view
├── lib/commands.ts           # Tauri invoke wrappers and shared types
└── stores/                   # Zustand stores
    ├── config.ts
    ├── engine.ts
    ├── format.ts
    └── history.ts

src-tauri/src/                # Backend source
├── commands/                 # Tauri command layer
│   ├── app_config.rs         # Load/save app configuration
│   ├── clipboard.rs          # Clipboard read/write
│   ├── engine_cmd.rs         # Engine status detection
│   ├── format_cmd.rs         # Format text
│   └── history_cmd.rs        # Query and clear history
├── config/app_config.rs      # AppConfig model and persistence
├── engine/
│   ├── types.rs              # FormatMode and FormatterEngine trait
│   └── autocorrect_cli.rs    # autocorrect CLI implementation
├── services/formatter.rs     # Formatting service
├── history_store/store.rs    # JSONL history storage
├── dto.rs                    # Data transfer objects
├── errors.rs                 # Shared app errors
└── lib.rs                    # App setup, tray, shortcuts, and autostart
```

## Formatting Modes

| Mode | Description |
|------|-------------|
| **Standard** | Adds spacing between CJK text and Latin letters or numbers, and normalizes full-width/half-width punctuation |
| **Strict** | Invokes `autocorrect --strict` and applies stricter rules on top of standard formatting |

## Configuration

App configuration is stored at:

- **macOS**: `~/Library/Application Support/cjk-autocorrect-desktop/config.json`
- **Windows**: `%APPDATA%/cjk-autocorrect-desktop/config.json`
- **Linux**: `~/.local/share/cjk-autocorrect-desktop/config.json`

History is stored as `history.jsonl` in the same directory.

## Privacy

CJK AutoCorrect Desktop is local-first and does not upload your text content.

- Formatting is performed locally by the `autocorrect` CLI.
- Clipboard read/write is triggered only by explicit user actions or formatting workflows.
- History is stored only on your device as `history.jsonl`, and unchanged text is not written to history.
- You can clear history from the app or delete the history file manually.
- The project does not include telemetry, account login, remote sync, or analytics.

See [PRIVACY.md](./PRIVACY.md) for details.

## Development Tools

Recommended IDE setup:

- [VS Code](https://code.visualstudio.com/)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) extension
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension
- [Tailwind CSS IntelliSense](https://marketplace.visualstudio.com/items?itemName=bradlc.vscode-tailwindcss) extension

## License

[MIT](./LICENSE)
