//! Host inspection: what this Mac is, and which compatibility tools are
//! actually present on it.
//!
//! Detection is passive. It looks for files and nothing more — it does not run
//! third-party binaries to interrogate them, and it never installs, downloads
//! or modifies anything. A capability reported here means "this was found on
//! disk", which is not a claim that any particular game will run.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::model::Capability;

/// macOS product version, e.g. `26.6.2`.
pub fn os_version() -> String {
    // Absolute path, argument array: no shell, no PATH lookup.
    Command::new("/usr/bin/sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|version| !version.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn arch() -> String {
    std::env::consts::ARCH.to_string()
}

/// Everything ReaPlay currently knows how to look for.
pub fn detect_capabilities() -> Vec<Capability> {
    vec![
        detect_rosetta(),
        detect_wine(),
        detect_game_porting_toolkit(),
    ]
}

fn detect_rosetta() -> Capability {
    // Rosetta 2 installs this runtime directory; its presence is the
    // documented way to tell it is available.
    let marker = Path::new("/Library/Apple/usr/libexec/oah");
    let available = marker.exists();
    Capability {
        id: "rosetta2".into(),
        display_name: "Rosetta 2".into(),
        available,
        version: None,
        detail: if available {
            None
        } else {
            Some("Not installed. Required by some x86-64 compatibility tools.".into())
        },
    }
}

fn detect_wine() -> Capability {
    let found = first_existing(&[
        "/opt/homebrew/bin/wine64",
        "/opt/homebrew/bin/wine",
        "/usr/local/bin/wine64",
        "/usr/local/bin/wine",
    ])
    .or_else(|| find_on_path("wine64"))
    .or_else(|| find_on_path("wine"));

    Capability {
        id: "wine".into(),
        display_name: "Wine".into(),
        available: found.is_some(),
        version: None,
        detail: match &found {
            Some(path) => Some(path.display().to_string()),
            None => Some("Not detected. ReaPlay does not install it for you.".into()),
        },
    }
}

fn detect_game_porting_toolkit() -> Capability {
    let found = first_existing(&[
        "/opt/homebrew/bin/gameportingtoolkit",
        "/usr/local/bin/gameportingtoolkit",
    ])
    .or_else(|| find_on_path("gameportingtoolkit"));

    Capability {
        id: "gptk".into(),
        display_name: "Game Porting Toolkit".into(),
        available: found.is_some(),
        version: None,
        detail: match &found {
            Some(path) => Some(path.display().to_string()),
            None => Some("Not detected. Apple distributes it under its own licence terms.".into()),
        },
    }
}

fn first_existing(candidates: &[&str]) -> Option<PathBuf> {
    candidates
        .iter()
        .map(Path::new)
        .find(|path| path.exists())
        .map(Path::to_path_buf)
}

fn find_on_path(binary: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .map(|dir| dir.join(binary))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_version_is_reported() {
        let version = os_version();
        assert!(!version.is_empty());
        assert_ne!(version, "unknown", "sw_vers should work on a macOS host");
    }

    #[test]
    fn arch_is_known() {
        assert!(matches!(arch().as_str(), "aarch64" | "x86_64"));
    }

    #[test]
    fn every_capability_explains_itself_when_unavailable() {
        for capability in detect_capabilities() {
            assert!(!capability.id.is_empty());
            if !capability.available {
                assert!(
                    capability.detail.is_some(),
                    "{} is unavailable but gives no reason",
                    capability.id
                );
            }
        }
    }
}
