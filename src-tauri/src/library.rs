//! Local persistence for game metadata.
//!
//! The library is a single JSON file in the app data directory. Writes go to a
//! temporary file in the same directory and are then renamed over the target,
//! so an interrupted write leaves the previous file intact rather than a
//! half-written one.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{IpcError, code};
use crate::model::Game;

/// Bump when the on-disk shape changes, and add a migration in `load`.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFile {
    pub schema_version: u32,
    pub games: Vec<Game>,
}

impl Default for LibraryFile {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            games: Vec::new(),
        }
    }
}

/// Read the library, recovering rather than failing when the file is damaged.
///
/// A file that will not parse is moved aside with a timestamped name and an
/// empty library is returned, which keeps the app usable without discarding
/// whatever the old file contained. A file written by a *newer* schema is an
/// error instead, because silently ignoring it would lose real data.
pub fn load(path: &Path) -> Result<LibraryFile, IpcError> {
    if !path.exists() {
        return Ok(LibraryFile::default());
    }

    let raw = fs::read_to_string(path).map_err(|e| {
        IpcError::detailed(
            code::STORE_UNREADABLE,
            "ReaPlay could not read your game library.",
            e.to_string(),
        )
    })?;

    match serde_json::from_str::<LibraryFile>(&raw) {
        Ok(file) if file.schema_version > SCHEMA_VERSION => Err(IpcError::detailed(
            code::STORE_CORRUPT,
            "Your game library was written by a newer version of ReaPlay.",
            format!(
                "file schema {} > supported {}",
                file.schema_version, SCHEMA_VERSION
            ),
        )),
        Ok(file) => Ok(file),
        Err(parse_error) => {
            let moved = quarantine(path);
            log::warn!(
                "library file unreadable ({parse_error}); moved aside to {moved:?}, starting empty"
            );
            Ok(LibraryFile::default())
        }
    }
}

/// Write the library atomically: temp file in the same directory, then rename.
pub fn save(path: &Path, file: &LibraryFile) -> Result<(), IpcError> {
    let parent = path.parent().ok_or_else(|| {
        IpcError::internal(format!("library path has no parent: {}", path.display()))
    })?;
    fs::create_dir_all(parent).map_err(|e| {
        IpcError::detailed(
            code::STORE_UNWRITABLE,
            "ReaPlay could not create its data folder.",
            e.to_string(),
        )
    })?;

    let body = serde_json::to_vec_pretty(file)
        .map_err(|e| IpcError::internal(format!("serialising library: {e}")))?;

    let temp = path.with_extension("json.tmp");
    {
        let mut handle = fs::File::create(&temp).map_err(|e| {
            IpcError::detailed(
                code::STORE_UNWRITABLE,
                "ReaPlay could not save your game library.",
                e.to_string(),
            )
        })?;
        handle.write_all(&body).map_err(|e| {
            IpcError::detailed(
                code::STORE_UNWRITABLE,
                "ReaPlay could not save your game library.",
                e.to_string(),
            )
        })?;
        // Flush to disk before the rename, so the rename cannot expose a
        // file whose contents have not landed yet.
        handle.sync_all().map_err(|e| {
            IpcError::detailed(
                code::STORE_UNWRITABLE,
                "ReaPlay could not save your game library.",
                e.to_string(),
            )
        })?;
    }

    fs::rename(&temp, path).map_err(|e| {
        let _ = fs::remove_file(&temp);
        IpcError::detailed(
            code::STORE_UNWRITABLE,
            "ReaPlay could not save your game library.",
            e.to_string(),
        )
    })
}

/// Move a damaged library file aside so it is preserved but out of the way.
fn quarantine(path: &Path) -> Option<PathBuf> {
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    let target = path.with_extension(format!("corrupt-{stamp}.json"));
    fs::rename(path, &target).ok().map(|_| target)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("reaplay-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn sample_game() -> Game {
        Game {
            id: uuid::Uuid::new_v4().to_string(),
            title: "Test".into(),
            executable_path: "/tmp/test.exe".into(),
            install_directory: None,
            provider_id: None,
            profile_id: None,
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn missing_file_loads_empty() {
        let path = temp_dir().join("library.json");
        let file = load(&path).expect("load");
        assert_eq!(file.schema_version, SCHEMA_VERSION);
        assert!(file.games.is_empty());
    }

    #[test]
    fn save_then_load_round_trips() {
        let path = temp_dir().join("library.json");
        let mut file = LibraryFile::default();
        file.games.push(sample_game());
        save(&path, &file).expect("save");

        let loaded = load(&path).expect("load");
        assert_eq!(loaded.games.len(), 1);
        assert_eq!(loaded.games[0].title, "Test");
    }

    #[test]
    fn save_leaves_no_temp_file_behind() {
        let dir = temp_dir();
        let path = dir.join("library.json");
        save(&path, &LibraryFile::default()).expect("save");
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .expect("read dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|name| name.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temp files left: {leftovers:?}");
    }

    #[test]
    fn corrupt_file_is_quarantined_and_library_recovers() {
        let dir = temp_dir();
        let path = dir.join("library.json");
        fs::write(&path, b"{ this is not json").expect("write garbage");

        let loaded = load(&path).expect("load should recover");
        assert!(loaded.games.is_empty());

        let quarantined = fs::read_dir(&dir)
            .expect("read dir")
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().contains("corrupt-"));
        assert!(quarantined, "damaged file should be preserved, not deleted");
    }

    #[test]
    fn newer_schema_is_an_error_not_a_silent_reset() {
        let path = temp_dir().join("library.json");
        fs::write(&path, br#"{"schemaVersion": 99, "games": []}"#).expect("write");
        let error = load(&path).expect_err("should refuse a newer schema");
        assert_eq!(error.code, code::STORE_CORRUPT);
    }
}
