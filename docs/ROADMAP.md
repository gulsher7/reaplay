# Roadmap

## Phase 0 — Repository foundation

- [x] Product scope and safety boundaries
- [x] High-level architecture
- [ ] Establish Tauri 2 + React + TypeScript scaffold
- [ ] Add formatting, linting, and CI

## Phase 1 — Read-only prototype

- [ ] App shell and navigation
- [ ] Empty library state
- [ ] Add/import a local game executable
- [ ] Persist metadata locally
- [ ] Rust capability and diagnostics commands
- [ ] Friendly errors and logs

## Phase 2 — Provider abstraction

- [ ] Provider interface and capability model
- [ ] Detect supported compatibility tools without changing the host system
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

Scope note: supporting a game is an empirical result, not a promise based on its name or store listing.
