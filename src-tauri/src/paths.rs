//! Validation for user-supplied filesystem paths.
//!
//! Every path arriving over IPC is untrusted. Nothing here executes anything;
//! it only decides whether a path is a real, readable file and resolves it to
//! a canonical form before it is stored.

use std::path::{Component, Path, PathBuf};

use crate::error::{IpcError, code};
use crate::model::Availability;

/// Validate a candidate game executable and return its canonical path.
///
/// Note that the executable bit is deliberately *not* required: a Windows
/// `.exe` sitting on a macOS volume usually has no POSIX execute permission,
/// and rejecting it on that basis would reject almost every real input.
pub fn validate_executable(raw: &str) -> Result<PathBuf, IpcError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(IpcError::new(code::PATH_INVALID, "No path was provided."));
    }
    if trimmed.contains('\0') {
        return Err(IpcError::new(
            code::PATH_INVALID,
            "That path contains an invalid character.",
        ));
    }

    let path = Path::new(trimmed);
    if !path.is_absolute() {
        return Err(IpcError::detailed(
            code::PATH_NOT_ABSOLUTE,
            "Please provide a full path, starting with “/”.",
            format!("received: {trimmed}"),
        ));
    }
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(IpcError::new(
            code::PATH_INVALID,
            "Please provide a path without “..” segments.",
        ));
    }

    // An unmounted external volume is a distinct, recoverable situation: the
    // game may be fine and the drive simply unplugged.
    if let Some(volume) = external_volume_root(path)
        && !volume.exists()
    {
        let name = volume
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| volume.display().to_string());
        return Err(IpcError::detailed(
            code::VOLUME_UNAVAILABLE,
            format!("The volume “{name}” is not connected."),
            volume.display().to_string(),
        ));
    }

    if !path.exists() {
        return Err(IpcError::detailed(
            code::PATH_NOT_FOUND,
            "No file exists at that path.",
            trimmed.to_string(),
        ));
    }
    if !path.is_file() {
        return Err(IpcError::detailed(
            code::PATH_NOT_A_FILE,
            "That path is a folder, not a game executable.",
            trimmed.to_string(),
        ));
    }

    // Reading the first byte is the only reliable permission check; metadata
    // alone does not tell us whether this process may open the file.
    std::fs::File::open(path).map_err(|e| {
        IpcError::detailed(
            code::PATH_UNREADABLE,
            "ReaPlay does not have permission to read that file.",
            e.to_string(),
        )
    })?;

    path.canonicalize().map_err(|e| {
        IpcError::detailed(
            code::PATH_INVALID,
            "That path could not be resolved.",
            e.to_string(),
        )
    })
}

/// `/Volumes/<name>` for a path on an external volume, otherwise `None`.
fn external_volume_root(path: &Path) -> Option<PathBuf> {
    let mut components = path.components();
    if components.next() != Some(Component::RootDir) {
        return None;
    }
    if components.next()?.as_os_str() != "Volumes" {
        return None;
    }
    let name = components.next()?;
    Some(Path::new("/Volumes").join(name))
}

/// Re-check a stored game's files without touching the library.
pub fn availability_of(executable_path: &str) -> Availability {
    let path = Path::new(executable_path);
    if let Some(volume) = external_volume_root(path)
        && !volume.exists()
    {
        return Availability::VolumeUnavailable;
    }
    if path.is_file() {
        Availability::Available
    } else {
        Availability::ExecutableMissing
    }
}

/// A reasonable default title derived from a file name: `Celeste.exe` becomes
/// `Celeste`, `my_cool_game.exe` becomes `My cool game`.
pub fn title_from_path(path: &Path) -> String {
    // A dotfile has no meaningful display name, and `file_stem` returns the
    // whole name for one (`.exe` stems to `.exe`, not to an empty string).
    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    if file_name.starts_with('.') {
        return "Untitled game".to_string();
    }

    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let cleaned = stem.replace(['_', '-'], " ").trim().to_string();
    if cleaned.is_empty() {
        return "Untitled game".to_string();
    }
    let mut chars = cleaned.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => cleaned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_file(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("reaplay-paths-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join(name);
        fs::write(&path, b"MZ").expect("write stub exe");
        path
    }

    #[test]
    fn empty_input_is_rejected() {
        let error = validate_executable("   ").expect_err("should reject");
        assert_eq!(error.code, code::PATH_INVALID);
    }

    #[test]
    fn relative_path_is_rejected() {
        let error = validate_executable("games/thing.exe").expect_err("should reject");
        assert_eq!(error.code, code::PATH_NOT_ABSOLUTE);
    }

    #[test]
    fn parent_traversal_is_rejected() {
        let error = validate_executable("/tmp/../etc/hosts").expect_err("should reject");
        assert_eq!(error.code, code::PATH_INVALID);
    }

    #[test]
    fn missing_file_is_rejected() {
        let error =
            validate_executable("/tmp/reaplay-does-not-exist-9e3f/game.exe").expect_err("reject");
        assert_eq!(error.code, code::PATH_NOT_FOUND);
    }

    #[test]
    fn directory_is_rejected() {
        let error = validate_executable("/tmp").expect_err("should reject a directory");
        assert_eq!(error.code, code::PATH_NOT_A_FILE);
    }

    #[test]
    fn unmounted_volume_is_reported_distinctly() {
        // A path shaped like an external volume that is not mounted must be
        // distinguishable from a genuinely missing file, so the UI can say
        // "reconnect the drive" instead of "file is gone".
        let error = validate_executable("/Volumes/ReaPlayNotMounted9e3f/game.exe")
            .expect_err("should reject");
        assert_eq!(error.code, code::VOLUME_UNAVAILABLE);
    }

    #[test]
    fn real_file_is_accepted_and_canonicalised() {
        let path = temp_file("game.exe");
        let resolved = validate_executable(path.to_str().expect("utf8")).expect("should accept");
        assert!(resolved.is_absolute());
        assert!(resolved.is_file());
    }

    #[test]
    fn a_windows_exe_without_the_execute_bit_is_still_accepted() {
        // The common real case: an .exe copied from a Windows volume has no
        // POSIX execute permission. Requiring one would reject valid input.
        let path = temp_file("no-exec-bit.exe");
        let mut perms = fs::metadata(&path).expect("metadata").permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o644);
        }
        fs::set_permissions(&path, perms).expect("set permissions");
        validate_executable(path.to_str().expect("utf8")).expect("should accept");
    }

    #[test]
    fn availability_distinguishes_missing_from_unmounted() {
        let path = temp_file("present.exe");
        assert!(matches!(
            availability_of(path.to_str().expect("utf8")),
            Availability::Available
        ));
        assert!(matches!(
            availability_of("/tmp/reaplay-gone-9e3f/game.exe"),
            Availability::ExecutableMissing
        ));
        assert!(matches!(
            availability_of("/Volumes/ReaPlayNotMounted9e3f/game.exe"),
            Availability::VolumeUnavailable
        ));
    }

    #[test]
    fn titles_are_derived_readably() {
        assert_eq!(title_from_path(Path::new("/g/Celeste.exe")), "Celeste");
        assert_eq!(
            title_from_path(Path::new("/g/my_cool_game.exe")),
            "My cool game"
        );
        assert_eq!(title_from_path(Path::new("/g/some-game.exe")), "Some game");
        assert_eq!(title_from_path(Path::new("/g/.exe")), "Untitled game");
    }
}
