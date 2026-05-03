/// Normalize user-facing shortcut aliases to the stable names expected by
/// `global-hotkey`.
pub fn normalize_shortcut(shortcut: &str) -> String {
    shortcut
        .split('+')
        .map(|part| match part.trim() {
            "Command" | "Cmd" => "Super",
            "Option" => "Alt",
            other => other,
        })
        .collect::<Vec<_>>()
        .join("+")
}
