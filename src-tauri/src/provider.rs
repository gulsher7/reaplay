//! Compatibility providers.
//!
//! A provider knows how to turn "run this Windows executable" into a concrete
//! program plus an argument array. It never builds a shell command string, and
//! it never installs or downloads anything: a provider that is not already
//! present on the host simply reports itself unavailable.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::diagnostics;
use crate::error::{IpcError, code};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub display_name: String,
    pub available: bool,
    /// Absolute path to the runtime binary, when one was found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_path: Option<String>,
    /// Why it is unavailable, when it is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// What a provider must be able to do before ReaPlay will launch through it.
pub trait Provider: Send + Sync {
    fn info(&self) -> ProviderInfo;

    /// The program and argument array to run `executable`.
    ///
    /// Returns `PROVIDER_UNAVAILABLE` rather than a best guess when the
    /// runtime is missing, so a launch fails before a process is spawned.
    fn command_for(&self, executable: &Path) -> Result<(PathBuf, Vec<String>), IpcError>;
}

/// Wine, in any of the layouts macOS actually puts it in.
pub struct WineProvider {
    runtime: Option<PathBuf>,
}

impl WineProvider {
    pub fn detect() -> Self {
        Self {
            runtime: diagnostics::find_wine_runtime(),
        }
    }

    /// Point a provider at a specific runtime. Lets the tests exercise the
    /// provider-to-process handoff without a real Wine installation.
    #[cfg(test)]
    pub fn with_runtime(runtime: PathBuf) -> Self {
        Self {
            runtime: Some(runtime),
        }
    }
}

impl Provider for WineProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "wine".into(),
            display_name: "Wine".into(),
            available: self.runtime.is_some(),
            runtime_path: self.runtime.as_ref().map(|p| p.display().to_string()),
            detail: match &self.runtime {
                Some(_) => None,
                None => Some("No Wine runtime found. ReaPlay does not install one for you.".into()),
            },
        }
    }

    fn command_for(&self, executable: &Path) -> Result<(PathBuf, Vec<String>), IpcError> {
        let runtime = self.runtime.clone().ok_or_else(|| {
            IpcError::new(
                code::PROVIDER_UNAVAILABLE,
                "No Wine runtime is installed, so this game cannot be launched.",
            )
        })?;
        // Argument array, never a command string: a path containing spaces or
        // quotes is data here, not syntax.
        Ok((runtime, vec![executable.display().to_string()]))
    }
}

/// Every provider ReaPlay knows about, detected fresh.
pub fn all() -> Vec<Box<dyn Provider>> {
    vec![Box::new(WineProvider::detect())]
}

pub fn by_id(id: &str) -> Result<Box<dyn Provider>, IpcError> {
    all()
        .into_iter()
        .find(|provider| provider.info().id == id)
        .ok_or_else(|| {
            IpcError::detailed(
                code::PROVIDER_NOT_FOUND,
                "That compatibility provider is not one ReaPlay knows about.",
                id.to_string(),
            )
        })
}

/// The provider a game should use: its pinned one, or the first available.
pub fn resolve(preferred: Option<&str>) -> Result<Box<dyn Provider>, IpcError> {
    if let Some(id) = preferred {
        return by_id(id);
    }
    all()
        .into_iter()
        .find(|provider| provider.info().available)
        .ok_or_else(|| {
            IpcError::new(
                code::PROVIDER_UNAVAILABLE,
                "No compatibility provider is installed on this Mac.",
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_provider_identifies_itself() {
        for provider in all() {
            let info = provider.info();
            assert!(!info.id.is_empty());
            assert!(!info.display_name.is_empty());
            if !info.available {
                assert!(info.detail.is_some(), "{} gives no reason", info.id);
            }
        }
    }

    #[test]
    fn unknown_provider_id_is_rejected() {
        // `Box<dyn Provider>` is not Debug, so `expect_err` is unavailable here.
        let error = by_id("definitely-not-a-provider")
            .err()
            .expect("should reject an unknown id");
        assert_eq!(error.code, code::PROVIDER_NOT_FOUND);
    }

    #[test]
    fn an_unavailable_wine_refuses_to_build_a_command() {
        let provider = WineProvider { runtime: None };
        let error = provider
            .command_for(Path::new("/tmp/game.exe"))
            .expect_err("should refuse without a runtime");
        assert_eq!(error.code, code::PROVIDER_UNAVAILABLE);
    }

    #[test]
    fn the_command_a_provider_builds_actually_runs() {
        // The seam between "provider decides what to run" and "launch runs it".
        // Using /bin/echo as the runtime means the argument array comes
        // straight back on stdout, so a mangled path would be visible.
        let provider = WineProvider::with_runtime(PathBuf::from("/bin/echo"));
        let executable = Path::new("/games/Some Game/game.exe");
        let (program, args) = provider.command_for(executable).expect("build command");

        let child = crate::launch::spawn_process(&program, &args, None).expect("spawn");
        let outcome = crate::launch::collect(child).expect("collect");
        assert_eq!(outcome.exit_code, Some(0));
        assert_eq!(outcome.stdout_tail.trim(), "/games/Some Game/game.exe");
    }

    #[test]
    fn a_present_runtime_produces_program_plus_argument_array() {
        let provider = WineProvider {
            runtime: Some(PathBuf::from("/usr/bin/true")),
        };
        let (program, args) = provider
            .command_for(Path::new("/games/My Game/game.exe"))
            .expect("should build a command");
        assert_eq!(program, PathBuf::from("/usr/bin/true"));
        // The space in the path must survive as one argument, not two.
        assert_eq!(args, vec!["/games/My Game/game.exe".to_string()]);
    }
}
