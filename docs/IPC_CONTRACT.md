# IPC contract

The single boundary between the React UI and the Rust application layer.

**Status: implemented through the launch pipeline.** The nine commands below
exist and are registered in `src-tauri/src/lib.rs`. The authority for each side is:

- Rust: `src-tauri/src/commands.rs`, `model.rs`, `error.rs`
- TypeScript: `src/ipc/types.ts`, `src/ipc/client.ts`

Nothing generates one from the other, so a change to either must be mirrored by
hand. Every type below is serialised as camelCase on the wire.

## Rules

- Every command returns `Result<T, IpcError>`; no command returns a bare string
  error.
- The UI renders backend-reported capability and never infers support itself.
- Paths crossing this boundary are untrusted until `src-tauri/src/paths.rs`
  validates them.
- Launch arguments will be argument arrays, never assembled command strings.
- Nothing is downloaded, installed or executed on the user's behalf.

## Commands

| Command | Args | Returns |
|---|---|---|
| `list_games` | — | `GameEntry[]` |
| `import_game` | `{ request: ImportGameRequest }` | `GameEntry` |
| `remove_game` | `{ id: string }` | `void` |
| `get_diagnostics` | — | `Diagnostics` |
| `list_providers` | — | `ProviderInfo[]` |
| `export_diagnostics` | — | `string` |
| `launch_game` | `{ id: string }` | `LaunchRecord` |
| `terminate_launch` | `{ launchId: string }` | `void` |
| `list_launches` | — | `LaunchRecord[]` |

`remove_game` forgets metadata only; it never deletes game files.

`launch_game` returns as soon as the process is running, with a record whose
`status` is `running`. The finished record arrives later on an event.

`export_diagnostics` returns a plain-text report with home-directory paths
replaced by `~`. It is assembled from host facts, detection results and launch
metadata only; a test asserts the home directory never appears in the output.

## Events

| Event | Payload |
|---|---|
| `reaplay://launch-finished` | `LaunchRecord` |

Emitted once per launch, after the outcome has been written to history.

## Error shape

```ts
type IpcError = {
  code: string;      // stable, machine-readable
  message: string;   // human-readable, safe to display
  detail?: string;   // optional diagnostic, may be verbose
};
```

Codes are declared once in `src-tauri/src/error.rs` (`mod code`) and mirrored in
`IpcErrorCode`. A code's string is part of this contract and must not change:

`PATH_NOT_ABSOLUTE`, `PATH_NOT_FOUND`, `PATH_NOT_A_FILE`, `PATH_INVALID`,
`PATH_UNREADABLE`, `VOLUME_UNAVAILABLE`, `GAME_NOT_FOUND`,
`GAME_ALREADY_IMPORTED`, `STORE_UNREADABLE`, `STORE_UNWRITABLE`,
`STORE_CORRUPT`, `INTERNAL`.

Launch-related codes: `PROVIDER_UNAVAILABLE`, `PROVIDER_NOT_FOUND`,
`GAME_UNAVAILABLE`, `LAUNCH_FAILED`, `LAUNCH_NOT_RUNNING`.

`BACKEND_UNAVAILABLE` is raised by the TypeScript client, not by Rust, when the
page is running without a Tauri host.

## Types

```ts
type Game = {
  id: string;                  // stable UUID
  title: string;
  executablePath: string;      // canonicalised, absolute
  installDirectory?: string;
  providerId?: string;
  profileId?: string;
  createdAt: string;           // RFC 3339
  updatedAt: string;
};

// Re-checked on every read, never stored: an external volume can disappear
// between any two calls.
type Availability = 'available' | 'executableMissing' | 'volumeUnavailable';
type GameEntry = Game & { availability: Availability };

type Capability = {
  id: string;
  displayName: string;
  available: boolean;          // observed on this host, not inferred
  version?: string;
  detail?: string;             // why unavailable, when it is
};

type Diagnostics = {
  appVersion: string;
  osVersion: string;
  arch: string;
  capabilities: Capability[];
  libraryPath: string;
};

type ImportGameRequest = {
  executablePath: string;
  title?: string;              // omit to derive one from the file name
};

type ProviderInfo = {
  id: string;
  displayName: string;
  available: boolean;
  runtimePath?: string;        // the runtime binary, when one was found
  detail?: string;
};

type LaunchStatus = 'running' | 'exited' | 'failed' | 'terminated';

type LaunchRecord = {
  id: string;
  gameId: string;
  gameTitle: string;
  providerId: string;
  program: string;             // exactly what was executed
  args: string[];              // an array, never a command string
  startedAt: string;
  finishedAt?: string;
  status: LaunchStatus;
  exitCode?: number;
  stdoutTail?: string;         // bounded to 16 KiB
  stderrTail?: string;
  error?: string;
};
```

## Resolved decisions

These were the contract's open questions. They are settled as follows.

**Where the metadata store lives.** A single `library.json` in Tauri's
`app_data_dir()`. Its path is reported by `get_diagnostics` as `libraryPath` and
shown in Settings. The file carries a `schemaVersion`, and migrations belong to
`src-tauri/src/library.rs`.

**Robustness.** Writes go to a temp file in the same directory, are flushed with
`sync_all`, then renamed over the target, so an interrupted write leaves the
previous file intact. A file that will not parse is renamed to
`library.corrupt-<timestamp>.json` and an empty library is returned, which keeps
the app usable without discarding data. A file whose `schemaVersion` is *newer*
than the running app is a hard `STORE_CORRUPT` error instead, because ignoring
it would silently lose real data.

**Who supplies the title.** Both, with the UI winning. `import_game` accepts an
optional `title`; when it is absent or blank, Rust derives one from the file
stem (`my_cool_game.exe` → `My cool game`).

**Capability caching.** Not cached. `get_diagnostics` re-detects on every call.
Detection is a handful of `exists()` checks, so correctness is worth more than
the saved microseconds, and a tool installed while the app is open is picked up
by the Re-check button.

**Executable permission.** Deliberately *not* required. A Windows `.exe` on a
macOS volume usually has no POSIX execute bit, so requiring one would reject
almost every real input. Validation requires a readable regular file.

**Where launch history lives.** Its own `launches.json`, not the library file.
A launch is an event and a game is a record; mixing them would mean rewriting
the whole library on every process exit. History is capped at 100 entries,
newest first. Unlike the library, a damaged history file is discarded rather
than quarantined — it is troubleshooting data, not user data.

**How games are launched.** `Command` with an explicit program path and an
argument array. No shell is involved, so a path containing spaces, quotes or
semicolons is data rather than syntax; there is a test asserting that a
shell-metacharacter argument is not interpreted. The process runs with its
install directory as the working directory, and its output is drained on
dedicated threads because a child that fills its stdout pipe while nothing
reads it would block forever.

## Still open

- [ ] Whether `providerId` / `profileId` stay on `Game` or move to a separate
      per-game profile record (Phase 4 decides this).
- [ ] Whether a provider needs a per-game prefix/bottle concept, and whether
      that belongs in this contract or behind the provider interface.
