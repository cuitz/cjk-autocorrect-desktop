# Privacy

CJK AutoCorrect Desktop is designed as a local-first desktop tool. It does not upload your text content to a server.

## What the app processes

- Text you paste into the app.
- Clipboard content when you explicitly use clipboard-related actions or the global shortcut.
- Formatting preferences such as theme, default mode, shortcut, and autostart setting.
- Formatting history for text that was actually changed by the formatter.

## Where data is stored

Configuration and history are stored on your device:

- macOS: `~/Library/Application Support/cjk-autocorrect-desktop/`
- Windows: `%APPDATA%/cjk-autocorrect-desktop/`
- Linux: `~/.local/share/cjk-autocorrect-desktop/`

The history file is stored as `history.jsonl` in the same app data directory.

## Network usage

The app does not include telemetry, account login, cloud sync, or remote analytics. Formatting is performed by invoking the local `autocorrect` CLI through standard input.

## Clipboard access

Clipboard access is used only for user-facing formatting workflows, such as reading selected clipboard text and writing formatted text back. The app does not continuously monitor or upload clipboard content.

## History control

Only modified formatting results are saved to history. You can clear history from the app, or manually delete `history.jsonl` from the app data directory.

## Third-party dependency

The formatting engine is the local `autocorrect` command-line tool. Please review the `autocorrect` project if you want to understand its behavior in detail.
