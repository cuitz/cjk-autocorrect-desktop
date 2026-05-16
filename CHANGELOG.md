# Changelog

All notable changes to this project will be documented in this file.

## 0.1.3

### Fixed

- Diff highlighting: corrected the join index in the Hirschberg LCS recursion (`L1[k] + L2[m - k]` → `L1[k] + L2[k]`), which was producing suboptimal diffs for inputs that triggered the recursive branch and over-highlighting unchanged characters.
- Diff highlighting: normalized add/remove ordering before merging so recursive joins consistently collapse into a single "change" segment instead of dropping a colored segment.
- Clipboard formatting: the `format_clipboard` Tauri command now emits the same `ClipboardFormatEvent { original_text, formatted_text, changed }` payload as the global shortcut, keeping the frontend listener consistent across both code paths.
- Clipboard formatting: an empty / whitespace-only clipboard is now a silent no-op for both the global shortcut and the explicit command.
- Global shortcut: clipboard formatting performed via the global shortcut now persists to history (when enabled), matching the behaviour of the in-app format action.
- Settings: theme value sent to the frontend is now produced by an explicit `theme_to_string` match instead of relying on `Debug` derive output.

### Performance

- History store: `is_duplicate_of_last` now seeks the end of the file and reads only the trailing line instead of parsing the entire history on every format call.
- History store: `append` only rewrites the file when the on-disk line count actually exceeds the user's `history_limit`.
- Formatter engine: `autocorrect::config::load` is now skipped when the rule signature is unchanged from the previous call.

### Refactor

- Extracted the shared "read clipboard → format → write back → emit → persist history" flow into a single helper used by the global shortcut and the `format_clipboard` command (previously triplicated).
- Replaced an `unwrap()` on the default window icon with a clearer `expect` message.

## 0.1.0 - Unreleased

- Initial desktop formatter experience with Tauri, React, and Rust.
- Added CJK text formatting through the bundled `autocorrect` Rust engine.
- Added clipboard read/write, history records, theme settings, system tray, global shortcut, and autostart settings.
- Added app icon assets and release-ready project metadata.
- Removed the unused strict formatting mode from the UI, API, and documentation.
- Added frontend/backend settings for the bundled autocorrect engine rule toggles.
