# Roadmap

Status is recorded from what has actually been run on an M4 MacBook running
macOS 26.6.2, not from what is believed to work.

## Phase 0 — Repository foundation

- [x] Product scope and safety boundaries
- [x] High-level architecture
- [x] Establish Tauri 2 + React + TypeScript scaffold
- [x] Add formatting, linting, and CI — `.github/workflows/ci.yml` runs
      `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` and the
      frontend build on a macOS runner. The workflow has not yet run on
      GitHub, so it is unproven until the first push.

## Phase 1 — Read-only prototype

- [x] App shell and navigation
- [x] Empty library state
- [x] Add/import a local game executable
- [x] Persist metadata locally
- [x] Rust capability and diagnostics commands
- [x] Friendly errors and logs

Phase 1 is complete. The import, list and remove paths are covered by tests
that drive the real command logic against temporary files.

## Phase 2 — Provider abstraction

- [x] Provider interface and capability model (`src-tauri/src/provider.rs`) —
      two providers now: `CrossOverProvider` and `WineProvider`
- [x] Detect supported compatibility tools without changing the host system —
      Rosetta 2, Wine, CrossOver and Game Porting Toolkit, by file checks only;
      no third-party binary is executed to interrogate it
- [ ] Per-game profile metadata
- [x] Validate provider configuration before launch — a launch resolves a
      provider and re-checks the game's files before any process is spawned

CrossOver 26.3 is installed on this machine (signed by CodeWeavers, notarized,
accepted by Gatekeeper — no quarantine stripping or re-signing was needed), with
a `win10_64` bottle named `ReaPlay`.

CrossOver is **not** stock Wine and must not be driven as if it were. Its `wine`
is a Perl wrapper that needs `--bottle`, and `--cx-app` resolves against the
bottle's drive mappings rather than the native filesystem, so the executable is
passed as the final bare argument instead. `find_wine_runtime` therefore no
longer reports CrossOver, which would otherwise have produced a command that
could not run.

## Phase 3 — Launch and observability

- [x] Launch using explicit executable + argument arrays — no shell at any
      point, asserted by a test that a shell-metacharacter argument is not
      interpreted
- [x] Capture process exit status and logs — stdout/stderr drained on their own
      threads, bounded to 16 KiB each
- [x] Display launch history — `launches.json`, newest first, capped at 100
- [x] Provide a diagnostic export that excludes secrets and personal data by
      default — home paths become `~`, and a test asserts the home directory
      cannot appear in the output

Also implemented: terminating a running game, and marking a record that was
left `running` by a crash as stopped rather than showing it as live.

The pipeline is verified against real processes (`/bin/echo`, `/bin/sleep`),
including spawn, argument handling, exit codes, output capture and termination.

It is now also verified against **real Windows binaries**, by two tests marked
`#[ignore]` because they need CrossOver installed, which CI does not have. Run
them with `cargo test --lib -- --ignored --nocapture`:

- `a_real_windows_executable_runs_through_crossover` launches the bottle's own
  `cmd.exe` and asserts the captured stdout contains the Windows banner — proof
  that Windows code ran, not merely that a process started.
- `a_real_windows_game_launches_and_can_be_stopped` launches Anarch, asserts it
  is tracked as running, terminates it, and asserts the recorded outcome is
  `Terminated` rather than a clean `Exited`.

## Phase 4 — Compatibility verification

- [x] First real Windows game launched end to end — Anarch 1.01 (public domain)
      runs through ReaPlay on macOS 26.6.2 / aarch64 via CrossOver 26.3, and is
      tracked and terminable from the app
- [x] Verify graphics, input, audio, exit behaviour, logs and resource usage —
      all six confirmed for Anarch; see `docs/FIRST_GAME.md`
- [ ] Test on supported macOS and Apple Silicon configurations
- [ ] Publish reproducible results per game and provider version
- [ ] Document unsupported features and known issues

Scope note: supporting a game is an empirical result, not a promise based on its
name or store listing.
