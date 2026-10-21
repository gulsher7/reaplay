# Handoff

Last updated: 2026-10-22 · branch `claude/phase-1-foundation`

## Current milestone

**Phase 1 — Foundation.** Functionally complete. The Tauri 2 + React + Rust
structure exists, the desktop app launches, navigation and the empty-library
state are in place, and the typed command surface with structured errors is
implemented on both sides.

## Implemented

**Rust (`src-tauri/src/`)**

| Module | Responsibility |
|---|---|
| `error.rs` | `IpcError` and the stable `code` constants |
| `model.rs` | `Game`, `Capability`, `Diagnostics`, `Availability`, `GameEntry` |
| `paths.rs` | Validation of untrusted paths; title derivation; availability checks |
| `library.rs` | Atomic JSON persistence, corruption recovery, schema versioning |
| `diagnostics.rs` | Host detection (macOS version, arch) and passive tool detection |
| `commands.rs` | The four Tauri commands and shared `AppState` |

**TypeScript (`src/`)**

- `ipc/` — typed client mirroring the Rust models; nothing else calls `invoke`
- `hooks/` — `useLibrary`, `useDiagnostics`
- `components/` — `Sidebar`, `TopBar`, `GameCard`, `ImportGameModal`,
  `ErrorNotice`, `BackendBanner`
- `views/` — `LibraryView`, `CompatibilityView`, `SettingsView`, `PlannedView`

## Commands executed

```
rustup (stable-aarch64-apple-darwin, minimal) + rustfmt + clippy
npm install -D @tauri-apps/cli@^2 ; npm install @tauri-apps/api@^2
npx tauri init --ci --app-name reaplay --frontend-dist ../dist ...
npm run build
cd src-tauri && cargo test --lib
cd src-tauri && cargo clippy --all-targets -- -D warnings
cd src-tauri && cargo fmt
npm run tauri dev
```

## Build and test results

- `npm run build` — **pass**, exit 0 (`tsc -b && vite build`, 1597 modules)
- `cargo test --lib` — **18 passed, 0 failed**
- `cargo clippy --all-targets -- -D warnings` — **clean**
- `cargo fmt` — applied

## Verified on the actual Mac

macOS 26.6.2, Apple M4, 16 GB.

- The desktop app builds and opens a native window.
- `get_diagnostics` returns real host data and the UI renders it: macOS 26.6.2,
  `aarch64`, ReaPlay 0.1.0, **Rosetta 2 detected**, Wine not found, Game Porting
  Toolkit not found.
- Startup log confirms the library loads from
  `~/Library/Application Support/dev.reaplay.app/library.json`.
- Sections that are not built show a `Planned` badge; no dead controls.

## Not yet verified by hand

These are implemented and unit-tested, but have **not** been exercised through
the running window:

- `import_game` with a real `.exe`, including the derived-title path
- `remove_game`
- Error rendering for a bad path (`PATH_NOT_FOUND`, `VOLUME_UNAVAILABLE`, …)
- External-volume behaviour against a physically disconnected Samsung T7

## Known bugs and blockers

- **No blockers.** Rust was missing at the start of the session and is now
  installed (1.99.1).
- No CI workflow exists; lint, test and build are local only.
- `package-lock.json` is untracked and should be committed for reproducible
  installs.
- A dotfile-named executable (`.exe`) used to produce the title `.exe`; fixed,
  with a regression test (`titles_are_derived_readably`).

## Next recommended action

**Phase 2 research before code.** Per the project brief, the compatibility
runtime should be chosen from primary sources — Apple Silicon support, macOS 26
compatibility, licensing and redistribution terms — *before* a provider
abstraction is written. Detection for Wine and the Game Porting Toolkit already
exists, so the research determines which one the abstraction is shaped around.

The cheap parallel task is hand-testing the import flow with any small Windows
`.exe` to close the "not yet verified" list above.

## Approval required

- Nothing is committed. All Phase 1 work is uncommitted on
  `claude/phase-1-foundation`.
- Obtaining a test game involves a legal purchase and is explicitly the
  owner's decision.
