//! Launching games and watching them run.
//!
//! Processes are started from a program path plus an argument array. No shell
//! is involved at any point, so a path containing spaces, quotes or semicolons
//! is data rather than syntax.
//!
//! A running game is tracked so it can be terminated, and its output is read
//! on dedicated threads: a child that fills its stdout pipe while nothing
//! drains it would block forever.

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{IpcError, code};

/// How much of a game's output to keep. Enough to diagnose a failure, bounded
/// so a chatty process cannot exhaust memory.
const OUTPUT_TAIL_LIMIT: usize = 16 * 1024;

const POLL_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LaunchStatus {
    Running,
    Exited,
    /// The process could not be started at all.
    Failed,
    /// ReaPlay terminated it on request.
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRecord {
    pub id: String,
    pub game_id: String,
    pub game_title: String,
    pub provider_id: String,
    /// Exactly what was executed, recorded for reproducibility.
    pub program: String,
    pub args: Vec<String>,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    pub status: LaunchStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout_tail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_tail: Option<String>,
    /// Why the launch failed, when it did.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// The result of running a process to completion.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub status: LaunchStatus,
    pub exit_code: Option<i32>,
    pub stdout_tail: String,
    pub stderr_tail: String,
}

/// Processes ReaPlay started and has not yet reaped.
#[derive(Default)]
pub struct Registry {
    running: Mutex<HashMap<String, Child>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    fn insert(&self, id: String, child: Child) -> Result<(), IpcError> {
        self.lock()?.insert(id, child);
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, HashMap<String, Child>>, IpcError> {
        self.running
            .lock()
            .map_err(|_| IpcError::internal("launch registry lock poisoned"))
    }

    /// Only the tests need to ask this directly; the UI reads running state
    /// from `running_ids` via `list_launches`.
    #[cfg(test)]
    pub fn is_running(&self, id: &str) -> bool {
        self.lock().map(|map| map.contains_key(id)).unwrap_or(false)
    }

    pub fn running_ids(&self) -> Vec<String> {
        self.lock()
            .map(|map| map.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Ask a running game to stop. Returns `LAUNCH_NOT_RUNNING` if it already
    /// exited, which is not an error the user needs to see as a failure.
    pub fn terminate(&self, id: &str) -> Result<(), IpcError> {
        let mut map = self.lock()?;
        let child = map.get_mut(id).ok_or_else(|| {
            IpcError::detailed(
                code::LAUNCH_NOT_RUNNING,
                "That game is not running.",
                id.to_string(),
            )
        })?;
        child.kill().map_err(|e| {
            IpcError::detailed(
                code::LAUNCH_FAILED,
                "ReaPlay could not stop that game.",
                e.to_string(),
            )
        })
    }
}

/// Start a process. The child is returned unregistered; callers that want it
/// tracked should use [`start_tracked`].
pub fn spawn_process(
    program: &Path,
    args: &[String],
    cwd: Option<&Path>,
) -> Result<Child, IpcError> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Running from the game's own directory is what most Windows games expect
    // when they look for their data files.
    if let Some(dir) = cwd
        && dir.is_dir()
    {
        command.current_dir(dir);
    }

    command.spawn().map_err(|e| {
        IpcError::detailed(
            code::LAUNCH_FAILED,
            "ReaPlay could not start that program.",
            format!("{}: {e}", program.display()),
        )
    })
}

/// Read a child's output to completion and wait for it to exit.
///
/// The app always launches through [`start_tracked`], which must not block;
/// this synchronous path exists so the tests can assert on process behaviour
/// without a polling loop.
#[cfg(test)]
pub fn collect(mut child: Child) -> Result<Outcome, IpcError> {
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());

    let status = child.wait().map_err(|e| {
        IpcError::detailed(
            code::LAUNCH_FAILED,
            "ReaPlay lost track of that game's process.",
            e.to_string(),
        )
    })?;

    Ok(Outcome {
        status: LaunchStatus::Exited,
        exit_code: status.code(),
        stdout_tail: stdout.join(),
        stderr_tail: stderr.join(),
    })
}

/// Start a process, track it for cancellation, and call `on_finish` with the
/// outcome once it exits. Returns as soon as the process is running.
pub fn start_tracked<F>(
    registry: Arc<Registry>,
    id: String,
    program: &Path,
    args: &[String],
    cwd: Option<&Path>,
    on_finish: F,
) -> Result<(), IpcError>
where
    F: FnOnce(Outcome) + Send + 'static,
{
    let mut child = spawn_process(program, args, cwd)?;
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    registry.insert(id.clone(), child)?;

    std::thread::spawn(move || {
        // Poll rather than block on `wait`, so the registry lock stays free
        // for a terminate request arriving from the UI.
        let outcome = loop {
            let finished = {
                match registry.lock() {
                    Ok(mut map) => match map.get_mut(&id) {
                        Some(child) => child.try_wait().ok().flatten(),
                        // Removed from under us: nothing left to wait for.
                        None => break None,
                    },
                    Err(_) => break None,
                }
            };
            if let Some(status) = finished {
                if let Ok(mut map) = registry.lock() {
                    map.remove(&id);
                }
                break Some(status);
            }
            std::thread::sleep(POLL_INTERVAL);
        };

        let (status, exit_code) = match outcome {
            // A process killed by a signal reports no exit code on Unix.
            Some(status) => match status.code() {
                Some(code) => (LaunchStatus::Exited, Some(code)),
                None => (LaunchStatus::Terminated, None),
            },
            None => (LaunchStatus::Terminated, None),
        };

        on_finish(Outcome {
            status,
            exit_code,
            stdout_tail: stdout.join(),
            stderr_tail: stderr.join(),
        });
    });

    Ok(())
}

/// A reader thread accumulating a bounded tail of one pipe.
struct Drain {
    handle: Option<std::thread::JoinHandle<String>>,
}

impl Drain {
    fn join(self) -> String {
        self.handle
            .and_then(|handle| handle.join().ok())
            .unwrap_or_default()
    }
}

fn drain<R: Read + Send + 'static>(source: Option<R>) -> Drain {
    let Some(mut source) = source else {
        return Drain { handle: None };
    };
    let handle = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            match source.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(read) => {
                    buffer.extend_from_slice(&chunk[..read]);
                    // Keep only the tail, so a long-running chatty process
                    // cannot grow this without bound.
                    if buffer.len() > OUTPUT_TAIL_LIMIT * 2 {
                        let cut = buffer.len() - OUTPUT_TAIL_LIMIT;
                        buffer.drain(..cut);
                    }
                }
            }
        }
        if buffer.len() > OUTPUT_TAIL_LIMIT {
            let cut = buffer.len() - OUTPUT_TAIL_LIMIT;
            buffer.drain(..cut);
        }
        String::from_utf8_lossy(&buffer).to_string()
    });
    Drain {
        handle: Some(handle),
    }
}

/// Directory a game should run from: its recorded install directory, falling
/// back to the executable's parent.
pub fn working_directory(executable: &Path, install_directory: Option<&str>) -> Option<PathBuf> {
    install_directory
        .map(PathBuf::from)
        .filter(|dir| dir.is_dir())
        .or_else(|| executable.parent().map(Path::to_path_buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_stdout_and_a_zero_exit() {
        let child = spawn_process(Path::new("/bin/echo"), &["hello".to_string()], None)
            .expect("spawn echo");
        let outcome = collect(child).expect("collect");
        assert_eq!(outcome.exit_code, Some(0));
        assert_eq!(outcome.stdout_tail.trim(), "hello");
    }

    #[test]
    fn reports_a_non_zero_exit() {
        let child = spawn_process(Path::new("/usr/bin/false"), &[], None).expect("spawn false");
        let outcome = collect(child).expect("collect");
        assert_eq!(outcome.exit_code, Some(1));
    }

    #[test]
    fn an_argument_containing_spaces_stays_one_argument() {
        // The whole point of using an argument array: this must not split.
        let child = spawn_process(
            Path::new("/bin/echo"),
            &["-n".to_string(), "/games/My Game/game.exe".to_string()],
            None,
        )
        .expect("spawn echo");
        let outcome = collect(child).expect("collect");
        assert_eq!(outcome.stdout_tail, "/games/My Game/game.exe");
    }

    #[test]
    fn an_argument_that_looks_like_shell_syntax_is_not_interpreted() {
        let payload = "; touch /tmp/reaplay-should-not-exist; echo";
        let child = spawn_process(
            Path::new("/bin/echo"),
            &["-n".to_string(), payload.to_string()],
            None,
        )
        .expect("spawn echo");
        let outcome = collect(child).expect("collect");
        assert_eq!(outcome.stdout_tail, payload);
        assert!(
            !Path::new("/tmp/reaplay-should-not-exist").exists(),
            "argument was interpreted by a shell"
        );
    }

    #[test]
    fn a_missing_program_fails_before_anything_runs() {
        let error = spawn_process(Path::new("/nonexistent/reaplay-9e3f"), &[], None)
            .expect_err("should fail");
        assert_eq!(error.code, code::LAUNCH_FAILED);
    }

    #[test]
    fn a_tracked_process_reports_its_exit() {
        let registry = Arc::new(Registry::new());
        let (tx, rx) = std::sync::mpsc::channel();
        start_tracked(
            registry.clone(),
            "launch-1".into(),
            Path::new("/bin/echo"),
            &["tracked".to_string()],
            None,
            move |outcome| tx.send(outcome).expect("send outcome"),
        )
        .expect("start");

        let outcome = rx
            .recv_timeout(Duration::from_secs(10))
            .expect("outcome within 10s");
        assert_eq!(outcome.status, LaunchStatus::Exited);
        assert_eq!(outcome.exit_code, Some(0));
        assert_eq!(outcome.stdout_tail.trim(), "tracked");
        assert!(!registry.is_running("launch-1"), "should be reaped");
    }

    #[test]
    fn a_tracked_process_can_be_terminated() {
        let registry = Arc::new(Registry::new());
        let (tx, rx) = std::sync::mpsc::channel();
        start_tracked(
            registry.clone(),
            "launch-2".into(),
            Path::new("/bin/sleep"),
            &["30".to_string()],
            None,
            move |outcome| tx.send(outcome).expect("send outcome"),
        )
        .expect("start");

        // Give the poll loop a moment to see the process before killing it.
        std::thread::sleep(Duration::from_millis(300));
        assert!(registry.is_running("launch-2"));
        registry.terminate("launch-2").expect("terminate");

        let outcome = rx
            .recv_timeout(Duration::from_secs(10))
            .expect("outcome within 10s");
        assert_eq!(outcome.status, LaunchStatus::Terminated);
        assert!(!registry.is_running("launch-2"));
    }

    #[test]
    fn terminating_something_not_running_is_reported_clearly() {
        let registry = Registry::new();
        let error = registry.terminate("nope").expect_err("should fail");
        assert_eq!(error.code, code::LAUNCH_NOT_RUNNING);
    }
}
