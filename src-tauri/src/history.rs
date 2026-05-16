use crate::errors::{AppError, AppResult};
use crate::language::LanguageId;
use crate::language::PlatformFormat;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, RwLock};

// ------------------------------------------------------------------------------
// History entry
// ------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp: String,
    pub recognized_title: Option<String>,
    pub recognized_text: Option<String>,
    pub language: LanguageId,
    pub platform: PlatformFormat,
    pub model: String,
    pub result: String,
    pub user_note: Option<String>,
}

// ------------------------------------------------------------------------------
// History store
// ------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HistoryFile {
    version: u32,
    entries: Vec<HistoryEntry>,
}

impl Default for HistoryFile {
    fn default() -> Self {
        Self {
            version: 1,
            entries: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct HistoryStore {
    path: PathBuf,
    entries: RwLock<Vec<HistoryEntry>>,
    save_enabled: Mutex<bool>,
}

impl HistoryStore {
    pub fn load(path: PathBuf, save_enabled: bool) -> Self {
        let entries = match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<HistoryFile>(&contents) {
                Ok(file) => file.entries,
                Err(error) => {
                    tracing::warn!(path = %path.display(), %error, "Could not parse history file");
                    Vec::new()
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "Could not read history file");
                Vec::new()
            }
        };

        Self {
            path,
            entries: RwLock::new(entries),
            save_enabled: Mutex::new(save_enabled),
        }
    }

    pub fn is_enabled(&self) -> bool {
        *self
            .save_enabled
            .lock()
            .expect("save_enabled lock poisoned")
    }

    pub fn set_enabled(&self, enabled: bool) {
        *self
            .save_enabled
            .lock()
            .expect("save_enabled lock poisoned") = enabled;
    }

    pub fn list(&self) -> Vec<HistoryEntry> {
        self.entries.read().expect("entries lock poisoned").clone()
    }

    pub fn add(&self, entry: HistoryEntry) -> AppResult<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        {
            let mut entries = self.entries.write().expect("entries lock poisoned");
            entries.push(entry);
        }

        self.write_to_disk()
    }

    pub fn delete(&self, id: &str) -> AppResult<bool> {
        {
            let mut entries = self.entries.write().expect("entries lock poisoned");
            let before = entries.len();
            entries.retain(|e| e.id != id);
            if entries.len() == before {
                return Ok(false);
            }
        }

        self.write_to_disk()?;
        Ok(true)
    }

    pub fn clear(&self) -> AppResult<()> {
        {
            let mut entries = self.entries.write().expect("entries lock poisoned");
            entries.clear();
        }

        self.write_to_disk()
    }

    fn write_to_disk(&self) -> AppResult<()> {
        let file = HistoryFile {
            version: 1,
            entries: self.list(),
        };

        let parent = self.path.parent().ok_or(AppError::SettingsSaveFailed)?;
        fs::create_dir_all(parent).map_err(|error| {
            tracing::error!(path = %parent.display(), %error, "Could not create history directory");
            AppError::SettingsSaveFailed
        })?;

        let temp_path = self.path.with_extension("tmp");
        let json = serde_json::to_string_pretty(&file).map_err(|error| {
            tracing::error!(%error, "Could not serialize history file");
            AppError::SettingsSaveFailed
        })?;

        fs::write(&temp_path, json).map_err(|error| {
            tracing::error!(path = %temp_path.display(), %error, "Could not write history temp file");
            AppError::SettingsSaveFailed
        })?;

        fs::rename(&temp_path, &self.path).map_err(|error| {
            tracing::error!(path = %self.path.display(), %error, "Could not rename history file");
            AppError::SettingsSaveFailed
        })?;

        Ok(())
    }
}

// ------------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        std::env::current_dir()
            .expect("cwd")
            .join("target")
            .join("question-scan-tests")
            .join(format!("{name}-{nonce}"))
            .join("history.json")
    }

    fn sample_entry(id: &str) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            recognized_title: Some("Two Sum".to_string()),
            recognized_text: Some("Given an array of integers...".to_string()),
            language: LanguageId::Cpp20,
            platform: PlatformFormat::Acm,
            model: "gpt-4o-mini".to_string(),
            result: "class Solution { ... }".to_string(),
            user_note: None,
        }
    }

    #[test]
    fn load_missing_file_starts_empty() {
        let path = temp_path("history-missing");
        let store = HistoryStore::load(path.clone(), true);

        assert!(store.list().is_empty());
        assert!(store.is_enabled());
    }

    #[test]
    fn add_entry_when_enabled() {
        let path = temp_path("history-add");
        let store = HistoryStore::load(path.clone(), true);

        store.add(sample_entry("1")).expect("add");
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.list()[0].id, "1");
    }

    #[test]
    fn add_ignored_when_disabled() {
        let path = temp_path("history-disabled");
        let store = HistoryStore::load(path.clone(), false);

        store.add(sample_entry("1")).expect("add");
        assert!(store.list().is_empty());
    }

    #[test]
    fn delete_entry_removes_it() {
        let path = temp_path("history-delete");
        let store = HistoryStore::load(path.clone(), true);

        store.add(sample_entry("1")).expect("add");
        store.add(sample_entry("2")).expect("add");

        let deleted = store.delete("1").expect("delete");
        assert!(deleted);
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.list()[0].id, "2");
    }

    #[test]
    fn delete_missing_returns_false() {
        let path = temp_path("history-delete-missing");
        let store = HistoryStore::load(path.clone(), true);

        let deleted = store.delete("missing").expect("delete");
        assert!(!deleted);
    }

    #[test]
    fn clear_removes_all() {
        let path = temp_path("history-clear");
        let store = HistoryStore::load(path.clone(), true);

        store.add(sample_entry("1")).expect("add");
        store.add(sample_entry("2")).expect("add");

        store.clear().expect("clear");
        assert!(store.list().is_empty());
    }

    #[test]
    fn persists_across_reload() {
        let path = temp_path("history-persist");
        let store = HistoryStore::load(path.clone(), true);

        store.add(sample_entry("1")).expect("add");

        let reloaded = HistoryStore::load(path, true);
        assert_eq!(reloaded.list().len(), 1);
        assert_eq!(reloaded.list()[0].id, "1");
    }
}
