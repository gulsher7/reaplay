# Roadmap

Status is recorded from what has actually been run on an M4 MacBook running
macOS 26.6.2, not from what is believed to work.

## Phase 0 — Repository foundation

- [x] Product scope and safety boundaries
- [x] High-level architecture
- [x] Establish Tauri 2 + React + TypeScript scaffold
- [~] Add formatting, linting, and CI — `cargo fmt` and `cargo clippy -D warnings`
      and `tsc -b` all run clean locally; no CI workflow exists yet

## Phase 1 — Read-only prototype

- [x] App shell and navigation
- [x] Empty library state
- [x] Add/import a local game executable
- [x] Persist metadata locally
- [x] Rust capability and diagnostics commands
- [x] Friendly errors and logs

Phase 1 is functionally complete but has not been exercised by hand in the
running window — see "Verified" and "Not yet verified" in `HANDOFF.md`.

## Phase 2 — Provider abstraction

- [ ] Provider interface and capability model
- [ ] Detect supported compatibility tools without changing the host system
      (partial: Rosetta 2, Wine and Game Porting Toolkit are detected today,
      but behind ad-hoc checks rather than a provider interface)
- [ ] Per-game profile metadata
- [ ] Validate provider configuration before launch

## Phase 3 — Launch and observability

- [ ] Launch using explicit executable + argument arrays
- [ ] Capture process exit status and logs
- [ ] Display launch history
- [ ] Provide a diagnostic export that excludes secrets and personal data by default

## Phase 4 — Compatibility verification

- [ ] Test on supported macOS and Apple Silicon configurations
- [ ] Publish reproducible results per game and provider version
- [ ] Document unsupported features and known issues

Scope note: supporting a game is an empirical result, not a promise based on its
name or store listing.
