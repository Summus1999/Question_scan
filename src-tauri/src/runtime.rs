use crate::settings::{AiResultState, ScreenshotState, TrayStatus};
use serde::Serialize;
use std::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSnapshot {
    pub tray_status: TrayStatus,
    pub screenshot_state: ScreenshotState,
    pub ai_result_state: AiResultState,
    pub global_shortcut_registered: bool,
    pub global_shortcut_error: Option<String>,
    pub global_shortcut_trigger_count: u64,
}

impl Default for RuntimeSnapshot {
    fn default() -> Self {
        Self {
            tray_status: TrayStatus::Idle,
            screenshot_state: ScreenshotState::Idle,
            ai_result_state: AiResultState::Idle,
            global_shortcut_registered: false,
            global_shortcut_error: None,
            global_shortcut_trigger_count: 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct RuntimeStore {
    snapshot: RwLock<RuntimeSnapshot>,
    active_global_shortcut: RwLock<Option<String>>,
}

impl RuntimeStore {
    /// Returns a cloned runtime snapshot so command handlers never hold the lock.
    pub fn snapshot(&self) -> RuntimeSnapshot {
        self.snapshot.read().expect("runtime lock poisoned").clone()
    }

    /// Maps the tray phase into the visible runtime states used by the frontend shell.
    pub fn set_tray_status(&self, tray_status: TrayStatus) -> RuntimeSnapshot {
        let mut snapshot = self.snapshot.write().expect("runtime lock poisoned");
        snapshot.tray_status = tray_status;
        match tray_status {
            TrayStatus::Idle => {
                snapshot.screenshot_state = ScreenshotState::Idle;
                snapshot.ai_result_state = AiResultState::Idle;
            }
            TrayStatus::Screenshot => {
                snapshot.screenshot_state = ScreenshotState::Capturing;
                snapshot.ai_result_state = AiResultState::Idle;
            }
            TrayStatus::Recognizing => {
                snapshot.screenshot_state = ScreenshotState::Cropping;
                snapshot.ai_result_state = AiResultState::Idle;
            }
            TrayStatus::Generating => {
                snapshot.screenshot_state = ScreenshotState::Ready;
                snapshot.ai_result_state = AiResultState::Streaming;
            }
            TrayStatus::Complete => {
                snapshot.screenshot_state = ScreenshotState::Ready;
                snapshot.ai_result_state = AiResultState::Complete;
            }
            TrayStatus::Failed => {
                snapshot.screenshot_state = ScreenshotState::Failed;
                snapshot.ai_result_state = AiResultState::Failed;
            }
        }
        snapshot.clone()
    }

    /// Records which shortcut this process currently owns, if any.
    pub fn set_global_shortcut_active(&self, shortcut: Option<String>) -> RuntimeSnapshot {
        *self
            .active_global_shortcut
            .write()
            .expect("shortcut lock poisoned") = shortcut.clone();

        let mut snapshot = self.snapshot.write().expect("runtime lock poisoned");
        snapshot.global_shortcut_registered = shortcut.is_some();
        if shortcut.is_some() {
            snapshot.global_shortcut_error = None;
        }
        snapshot.clone()
    }

    /// Checks whether the requested shortcut is already owned by this process.
    pub fn is_global_shortcut_active(&self, shortcut: &str) -> bool {
        self.active_global_shortcut
            .read()
            .expect("shortcut lock poisoned")
            .as_deref()
            == Some(shortcut)
    }

    /// Stores the latest shortcut registration problem so the UI can show a clear prompt.
    pub fn set_global_shortcut_error(&self, error: Option<String>) -> RuntimeSnapshot {
        if error.is_some() {
            *self
                .active_global_shortcut
                .write()
                .expect("shortcut lock poisoned") = None;
        }

        let mut snapshot = self.snapshot.write().expect("runtime lock poisoned");
        snapshot.global_shortcut_error = error;
        if snapshot.global_shortcut_error.is_some() {
            snapshot.global_shortcut_registered = false;
        }
        snapshot.clone()
    }

    /// Marks a user-triggered shortcut event without starting real capture before stage 3.
    pub fn record_global_shortcut_trigger(&self) -> RuntimeSnapshot {
        let mut snapshot = self.snapshot.write().expect("runtime lock poisoned");
        snapshot.global_shortcut_trigger_count += 1;
        snapshot.tray_status = TrayStatus::Screenshot;
        snapshot.screenshot_state = ScreenshotState::Capturing;
        snapshot.ai_result_state = AiResultState::Idle;
        snapshot.global_shortcut_error = None;
        snapshot.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_status_updates_related_runtime_states() {
        let store = RuntimeStore::default();

        let generating = store.set_tray_status(TrayStatus::Generating);
        assert_eq!(generating.tray_status, TrayStatus::Generating);
        assert_eq!(generating.screenshot_state, ScreenshotState::Ready);
        assert_eq!(generating.ai_result_state, AiResultState::Streaming);

        let failed = store.set_tray_status(TrayStatus::Failed);
        assert_eq!(failed.screenshot_state, ScreenshotState::Failed);
        assert_eq!(failed.ai_result_state, AiResultState::Failed);
    }

    #[test]
    fn shortcut_trigger_records_count_and_capture_placeholder() {
        let store = RuntimeStore::default();

        let snapshot = store.record_global_shortcut_trigger();

        assert_eq!(snapshot.global_shortcut_trigger_count, 1);
        assert_eq!(snapshot.tray_status, TrayStatus::Screenshot);
        assert_eq!(snapshot.screenshot_state, ScreenshotState::Capturing);
    }

    #[test]
    fn active_shortcut_tracks_process_owned_registration() {
        let store = RuntimeStore::default();

        let registered = store.set_global_shortcut_active(Some("Ctrl+Shift+Q".to_string()));
        assert!(registered.global_shortcut_registered);
        assert!(store.is_global_shortcut_active("Ctrl+Shift+Q"));

        let failed = store.set_global_shortcut_error(Some("conflict".to_string()));
        assert!(!failed.global_shortcut_registered);
        assert!(!store.is_global_shortcut_active("Ctrl+Shift+Q"));
    }
}
