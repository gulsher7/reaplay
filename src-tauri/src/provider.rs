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
use crate::launch::StopCommand;

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

    /// How to stop a game this provider started, when killing the spawned
    /// process does not stop it.
    ///
    /// `None` means the spawned process *is* the game, so killing it is
    /// enough. Returning a command here is not an optimisation: for Wine the
    /// game survives otherwise, and ReaPlay would report a stop that did not
    /// happen.
    fn stop_command(&self) -> Option<StopCommand> {
        None
    }
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

/// CrossOver, which wraps Wine in a bottle-aware Perl launcher.
///
/// Its CLI is not stock Wine's: the executable must be passed as the first
/// non-option argument (`--cx-app` resolves against the bottle's drive
/// mappings and rejects a native path), and a `--bottle` is mandatory because
/// the implicit `default` bottle usually does not exist.
pub struct CrossOverProvider {
    runtime: Option<PathBuf>,
    bottle: Option<String>,
}

/// The bottle ReaPlay creates and prefers, when the user has several.
const PREFERRED_BOTTLE: &str = "ReaPlay";

impl CrossOverProvider {
    pub fn detect() -> Self {
        let bottles = diagnostics::crossover_bottles();
        let bottle = bottles
            .iter()
            .find(|name| name.as_str() == PREFERRED_BOTTLE)
            .or_else(|| bottles.first())
            .cloned();
        Self {
            runtime: diagnostics::find_crossover_runtime(),
            bottle,
        }
    }
}

impl Provider for CrossOverProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "crossover".into(),
            display_name: "CrossOver".into(),
            available: self.runtime.is_some() && self.bottle.is_some(),
            runtime_path: self.runtime.as_ref().map(|p| p.display().to_string()),
            detail: match (&self.runtime, &self.bottle) {
                (Some(_), Some(bottle)) => Some(format!("bottle: {bottle}")),
                (Some(_), None) => {
                    Some("CrossOver is installed but has no bottle to launch into.".into())
                }
                (None, _) => Some("CrossOver is not installed.".into()),
            },
        }
    }

    /// `wineserver -k`, scoped to this bottle by `WINEPREFIX`.
    ///
    /// Measured, not assumed: killing the spawned `wine` wrapper leaves the
    /// game running, because Wine reparents it to init and gives it its own
    /// process group, so neither a child kill nor a process-group kill reaches
    /// it. `wineserver -k` does stop it.
    ///
    /// **This stops everything running in the bottle.** With one bottle per
    /// game that is exact; with a shared bottle, stopping one game stops the
    /// others in it too. Per-game bottles are the fix, not a wider kill.
    fn stop_command(&self) -> Option<StopCommand> {
        let runtime = self.runtime.as_ref()?;
        let bottle = self.bottle.as_ref()?;
        // `wineserver` sits next to `wine` in the CrossOver bundle.
        let wineserver = runtime.parent()?.join("wineserver");
        let prefix = PathBuf::from(std::env::var_os("HOME")?)
            .join("Library/Application Support/CrossOver/Bottles")
            .join(bottle);
        Some(StopCommand {
            program: wineserver,
            args: vec!["-k".to_string()],
            env: vec![("WINEPREFIX".to_string(), prefix.display().to_string())],
        })
    }

    fn command_for(&self, executable: &Path) -> Result<(PathBuf, Vec<String>), IpcError> {
        let runtime = self.runtime.clone().ok_or_else(|| {
            IpcError::new(
                code::PROVIDER_UNAVAILABLE,
                "CrossOver is not installed, so this game cannot be launched.",
            )
        })?;
        let bottle = self.bottle.clone().ok_or_else(|| {
            IpcError::new(
                code::PROVIDER_UNAVAILABLE,
                "CrossOver has no bottle configured, so this game cannot be launched.",
            )
        })?;
        Ok((
            runtime,
            vec![
                "--bottle".to_string(),
                bottle,
                // Errors belong in the launch log, not in a modal dialog that
                // nobody is there to dismiss.
                "--no-gui".to_string(),
                // Wait for the game, so the recorded exit code is the game's.
                "--wait".to_string(),
                // Last, and bare: CrossOver converts a native path itself.
                executable.display().to_string(),
            ],
        ))
    }
}

/// Every provider ReaPlay knows about, detected fresh.
///
/// CrossOver comes first: it is the signed, notarized, supported runtime, so
/// it is what `resolve` should pick when both are present.
pub fn all() -> Vec<Box<dyn Provider>> {
    vec![
        Box::new(CrossOverProvider::detect()),
        Box::new(WineProvider::detect()),
    ]
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
    fn crossover_without_a_bottle_refuses_to_build_a_command() {
        // Installed-but-bottle-less must fail before spawning, not produce a
        // command that falls back to a `default` bottle that does not exist.
        let provider = CrossOverProvider {
            runtime: Some(PathBuf::from("/usr/bin/true")),
            bottle: None,
        };
        let error = provider
            .command_for(Path::new("/games/game.exe"))
            .expect_err("should refuse without a bottle");
        assert_eq!(error.code, code::PROVIDER_UNAVAILABLE);
    }

    #[test]
    fn crossover_passes_the_bottle_and_a_bare_executable_path() {
        let provider = CrossOverProvider {
            runtime: Some(PathBuf::from("/usr/bin/true")),
            bottle: Some("ReaPlay".to_string()),
        };
        let (program, args) = provider
            .command_for(Path::new("/games/My Game/game.exe"))
            .expect("should build a command");
        assert_eq!(program, PathBuf::from("/usr/bin/true"));
        assert_eq!(
            args,
            vec![
                "--bottle".to_string(),
                "ReaPlay".to_string(),
                "--no-gui".to_string(),
                "--wait".to_string(),
                // One argument, space and all, and never behind `--cx-app`.
                "/games/My Game/game.exe".to_string(),
            ]
        );
    }

    #[test]
    fn crossover_supplies_a_bottle_scoped_shutdown_command() {
        let provider = CrossOverProvider {
            runtime: Some(PathBuf::from(
                "/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine",
            )),
            bottle: Some("ReaPlay".to_string()),
        };
        let stop = provider.stop_command().expect("a stop command");
        assert!(stop.program.ends_with("wineserver"));
        assert_eq!(stop.args, vec!["-k".to_string()]);
        // Scoped to the bottle: an unscoped `wineserver -k` would stop every
        // bottle on the machine.
        let (key, value) = &stop.env[0];
        assert_eq!(key, "WINEPREFIX");
        assert!(value.ends_with("/Bottles/ReaPlay"), "{value}");
    }

    #[test]
    fn bare_wine_needs_no_shutdown_command() {
        let provider = WineProvider {
            runtime: Some(PathBuf::from("/usr/bin/true")),
        };
        assert!(provider.stop_command().is_none());
    }

    #[test]
    fn crossover_is_preferred_over_bare_wine() {
        // Order matters: `resolve` takes the first available provider.
        let ids: Vec<String> = all().iter().map(|p| p.info().id).collect();
        assert_eq!(ids, vec!["crossover".to_string(), "wine".to_string()]);
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
