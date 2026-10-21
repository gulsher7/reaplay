# Handoff

Last updated: 2026-10-22 · branch `claude/phase-1-foundation`

## Read this first

The app is built and runs. **It cannot launch a Windows game yet, and that is
blocked on a decision only you can make** — see "The one thing I need from
you". Everything else on the path to a first game launch is done.

## Current milestone

Phase 1 complete. Phase 2 (provider abstraction) and Phase 3 (launch and
observability) are implemented apart from per-game profiles and diagnostic
export.

## What was built this session

**Phase 1 — foundation**

Tauri 2 + React + Rust structure, the typed command surface, structured errors,
library persistence, host diagnostics, navigation and honest empty states.

**Phase 2 — provider abstraction** (`src-tauri/src/provider.rs`)

A `Provider` trait that turns "run this executable" into a program path plus an
argument array. Wine is the only implementation. A provider that is not present
on the host reports itself unavailable; nothing is ever downloaded or installed.

**Phase 3 — launch and observability** (`src-tauri/src/launch.rs`, `history.rs`)

- Processes start from a program path and an argument array. No shell is
  involved, so a path containing spaces, quotes or semicolons is data, not
  syntax. There is a test asserting a shell-metacharacter argument is not
  interpreted.
- stdout and stderr are drained on dedicated threads and bounded to 16 KiB
  each. A child that fills its pipe while nothing reads it would otherwise
  block forever.
- Running games are tracked and can be stopped. The watcher polls rather than
  blocking, so a stop request is never queued behind a running game.
- Launch history lives in its own `launches.json`, newest first, capped at 100.
  A record left `running` by a crash is shown as stopped, not as live.

**Diagnostic export** (`src-tauri/src/report.rs`)

A plain-text report for bug reports: host facts, detected tools, providers,
library and recent launches. Home-directory paths become `~`, and a test
asserts the home directory cannot appear in the output. Generate and copy it
from Settings.

**CI** — `.github/workflows/ci.yml` runs `cargo fmt --check`, clippy with
`-D warnings`, the Rust tests and the frontend build on a macOS runner. It has
never run on GitHub, so treat it as unproven until the first push.

**UI** — a real Launch button (disabled, with the reason, when no provider is
installed) and a Launch logs view showing exactly what was executed, the exit
code, and the captured output.

## Test results — all actually run

```
cargo test --lib                            51 passed, 0 failed
cargo clippy --all-targets -- -D warnings   clean
cargo fmt                                   applied
npm run build (tsc -b && vite build)        pass, exit 0
```

The 51 include end-to-end tests that drive the real command logic against
temporary files: import → list → remove, import deduplication, persistence
across a restart, a game whose file vanished, and every launch refusal path.

## Verified on your Mac

macOS 26.6.2, Apple M4.

- The desktop app builds, launches, and loads its stores:
  `library: 0 game(s), 0 past launch(es)`.
- `get_diagnostics` renders real host data; Rosetta 2 detected, Wine, CrossOver
  and Game Porting Toolkit correctly not found.
- The launch pipeline works against real processes — `/bin/echo` and
  `/bin/sleep` — covering spawn, argument handling, exit codes, output capture
  and termination.

## Not verified

- A real Windows game. No compatibility runtime is installed.
- The UI by hand: I cannot click the window. Every path behind the buttons is
  covered by tests against the same functions the commands call.
- Your `ReaPlay-Test.exe` is a **1024-byte PE32+ header stub**, not a runnable
  program. It is fine for testing import and validation, but there is no code
  in it to execute, so it cannot prove a launch even once Wine is installed.

## The one thing I need from you

ReaPlay needs a Wine runtime. Every route has a catch, and I did not want to
make this call on your machine while you were asleep.

1. **Homebrew's own Wine casks are disabled.** `wine-stable` and `wine@staging`
   were disabled upstream on 2026-09-01: "does not pass the macOS Gatekeeper
   check". `brew install` will refuse.

2. **The Game Porting Toolkit cask (Gcenx tap) strips Gatekeeper.** Its
   `postflight_steps` run:
   ```
   /usr/bin/xattr -drs com.apple.quarantine "/Applications/Game Porting Toolkit.app"
   /usr/bin/codesign --force --deep -s - "/Applications/Game Porting Toolkit.app"
   ```
   That removes the quarantine flag and ad-hoc re-signs the app. It is also a
   third-party repackage of Apple's proprietary toolkit, which your own brief
   says not to redistribute or assume permission for.

3. **Sikarugir** requires `brew trust` on a third-party tap, is GUI-oriented
   with no documented CLI entry point, and bundles D3DMetal under a licence its
   own README calls restrictive and unusable for commercial ports.

4. **CrossOver** is commercial and paid. Version 26 bundles D3DMetal 3.0 and
   DXMT. It is the only route that is signed, supported and licensed.

ReaPlay already detects all four, so whichever you choose is picked up with no
code change. My recommendation: if this is a personal experiment, option 2 is
what the Mac gaming community actually uses, and you should make that
Gatekeeper decision knowingly. If it ever ships, option 4 is the only
defensible one.

## Also needed

A real test game. Your brief says a purchase is your decision, so I have not
obtained one. Once a runtime is in place, any small, DRM-free Windows game is
enough — an itch.io freeware title avoids both the cost and the DRM question.

## Known gaps

- Per-game provider profiles are not implemented (Phase 2 remainder).
- The CI workflow is written but has never executed; the first push will be
  its first real test.
- The CI YAML could not be machine-validated locally (no YAML parser on this
  machine), only checked structurally.

## Next recommended action

Pick a runtime from the four above. Then: install it, confirm it appears under
Compatibility, import a small real game, and press Launch. The launch logs will
show exactly what ran and what came back, which is what makes the first failure
diagnosable rather than mysterious.
