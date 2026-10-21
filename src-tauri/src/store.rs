//! Durable-write helper shared by the on-disk stores.
//!
//! Writes go to a temporary file in the same directory, are flushed to disk,
//! and are then renamed over the target. An interrupted write therefore leaves
//! the previous file intact rather than a half-written one.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::error::{IpcError, code};

pub fn atomic_write(path: &Path, body: &[u8], what: &str) -> Result<(), IpcError> {
    let parent = path.parent().ok_or_else(|| {
        IpcError::internal(format!("store path has no parent: {}", path.display()))
    })?;
    fs::create_dir_all(parent).map_err(|e| {
        IpcError::detailed(
            code::STORE_UNWRITABLE,
            "ReaPlay could not create its data folder.",
            e.to_string(),
        )
    })?;

    let failure = |e: std::io::Error| {
        IpcError::detailed(
            code::STORE_UNWRITABLE,
            format!("ReaPlay could not save your {what}."),
            e.to_string(),
        )
    };

    let temp = path.with_extension("json.tmp");
    {
        let mut handle = fs::File::create(&temp).map_err(failure)?;
        handle.write_all(body).map_err(failure)?;
        // Flush before the rename, so the rename cannot publish a file whose
        // contents have not reached disk yet.
        handle.sync_all().map_err(failure)?;
    }

    fs::rename(&temp, path).map_err(|e| {
        let _ = fs::remove_file(&temp);
        failure(e)
    })
}
