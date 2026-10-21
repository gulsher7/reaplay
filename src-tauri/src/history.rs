//! Launch history.
//!
//! Kept in its own file rather than inside the library, because a launch is an
//! event and a game is a record: mixing them would mean rewriting the whole
//! library on every process exit.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{IpcError, code};
use crate::launch::LaunchRecord;

pub const SCHEMA_VERSION: u32 = 1;

/// Oldest entries are dropped past this; history is for troubleshooting the
/// recent past, not an audit log.
pub const HISTORY_LIMIT: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryFile {
    pub schema_version: u32,
    pub launches: Vec<LaunchRecord>,
}

impl Default for HistoryFile {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            launches: Vec::new(),
        }
    }
}

impl HistoryFile {
    /// Add a record, newest first, trimming to [`HISTORY_LIMIT`].
    pub fn push(&mut self, record: LaunchRecord) {
        self.launches.insert(0, record);
        self.launches.truncate(HISTORY_LIMIT);
    }

    /// Replace a record in place, matching on id. Returns whether it was found.
    pub fn replace(&mut self, record: LaunchRecord) -> bool {
        match self
            .launches
            .iter_mut()
            .find(|existing| existing.id == record.id)
        {
            Some(existing) => {
                *existing = record;
                true
            }
            None => false,
        }
    }
}

/// Read the history. Damaged history is not worth failing a launch over, so an
/// unparsable file is discarded rather than quarantined.
pub fn load(path: &Path) -> Result<HistoryFile, IpcError> {
    if !path.exists() {
        return Ok(HistoryFile::default());
    }
    let raw = fs::read_to_string(path).map_err(|e| {
        IpcError::detailed(
            code::STORE_UNREADABLE,
            "ReaPlay could not read your launch history.",
            e.to_string(),
        )
    })?;
    match serde_json::from_str::<HistoryFile>(&raw) {
        Ok(file) => Ok(file),
        Err(parse_error) => {
            log::warn!("launch history unreadable ({parse_error}); starting empty");
            Ok(HistoryFile::default())
        }
    }
}

pub fn save(path: &Path, file: &HistoryFile) -> Result<(), IpcError> {
    let body = serde_json::to_vec_pretty(file)
        .map_err(|e| IpcError::internal(format!("serialising history: {e}")))?;
    crate::store::atomic_write(path, &body, "launch history")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::launch::LaunchStatus;

    fn record(id: &str) -> LaunchRecord {
        LaunchRecord {
            id: id.into(),
            game_id: "game".into(),
            game_title: "Game".into(),
            provider_id: "wine".into(),
            program: "/usr/bin/true".into(),
            args: vec![],
            started_at: "2026-01-01T00:00:00Z".into(),
            finished_at: None,
            status: LaunchStatus::Running,
            exit_code: None,
            stdout_tail: None,
            stderr_tail: None,
            error: None,
        }
    }

    #[test]
    fn newest_launch_comes_first() {
        let mut file = HistoryFile::default();
        file.push(record("one"));
        file.push(record("two"));
        assert_eq!(file.launches[0].id, "two");
    }

    #[test]
    fn history_is_capped() {
        let mut file = HistoryFile::default();
        for n in 0..(HISTORY_LIMIT + 25) {
            file.push(record(&format!("launch-{n}")));
        }
        assert_eq!(file.launches.len(), HISTORY_LIMIT);
        // The newest survives, the oldest is dropped.
        assert_eq!(
            file.launches[0].id,
            format!("launch-{}", HISTORY_LIMIT + 24)
        );
    }

    #[test]
    fn replace_updates_in_place() {
        let mut file = HistoryFile::default();
        file.push(record("one"));
        let mut finished = record("one");
        finished.status = LaunchStatus::Exited;
        finished.exit_code = Some(0);
        assert!(file.replace(finished));
        assert_eq!(file.launches.len(), 1);
        assert_eq!(file.launches[0].status, LaunchStatus::Exited);
    }

    #[test]
    fn replace_reports_a_missing_record() {
        let mut file = HistoryFile::default();
        assert!(!file.replace(record("ghost")));
    }

    #[test]
    fn unreadable_history_starts_empty_rather_than_failing() {
        let dir = std::env::temp_dir().join(format!("reaplay-history-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("launches.json");
        fs::write(&path, b"not json at all").expect("write garbage");
        let loaded = load(&path).expect("should recover");
        assert!(loaded.launches.is_empty());
    }
}
