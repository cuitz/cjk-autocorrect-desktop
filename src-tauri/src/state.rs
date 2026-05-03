use std::sync::atomic::AtomicBool;

pub struct AppRuntimeState {
    pub close_to_tray: AtomicBool,
}

impl AppRuntimeState {
    pub fn new(close_to_tray: bool) -> Self {
        Self {
            close_to_tray: AtomicBool::new(close_to_tray),
        }
    }
}
