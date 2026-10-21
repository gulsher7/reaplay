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

- [x] Provider interface and capability model (`src-tauri/src/provider.rs`)
- [x] Detect supported compatibility tools without changing the host system —
      Rosetta 2, Wine, CrossOver and Game Porting Toolkit, by file checks only;
      no third-party binary is executed to interrogate it
- [ ] Per-game profile metadata
- [x] Validate provider configuration before launch — a launch resolves a
      provider and re-checks the game's files before any process is spawned

Only one provider (Wine) is implemented. **No Wine runtime is installed on this
machine**, so the provider reports itself unavailable and launching is refused
with `PROVIDER_UNAVAILABLE` rather than failing obscurely.

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
It has **not** been verified against a real Windows game, because no
compatibility runtime is installed.

## Phase 4 — Compatibility verification

- [ ] Test on supported macOS and Apple Silicon configurations
- [ ] Publish reproducible results per game and provider version
- [ ] Document unsupported features and known issues

Scope note: supporting a game is an empirical result, not a promise based on its
name or store listing.
