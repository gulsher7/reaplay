# ReaPlay

**An experimental macOS launcher for Windows games on Apple Silicon.**

The project will start with a small, runnable UI prototype and one carefully chosen test game. We will validate each layer before attempting larger or more demanding titles. ReaPlay is independent of Highball; it will not depend on Highball being installed.

## Current status

- [x] Product scope and architecture notes
- [x] Initial interactive React + TypeScript UI prototype
- [x] Run the UI locally and refine the flow
- [x] Add Tauri 2 and the native Rust backend
- [x] Detect host capabilities and installed compatibility tools
- [x] Import a lawfully obtained local game (implemented; not yet exercised by hand)
- [ ] Test a small game and record reproducible results
- [ ] Expand compatibility only after the first test works

**Important:** ReaPlay cannot launch a Windows game yet. It manages a library of
local executables and reports what compatibility tooling this Mac has. The
compatibility provider layer does not exist. Anything not implemented is
labelled *Planned* in the interface rather than stubbed with a dead control.

## Prototype stack

- Tauri 2 (desktop shell)
- React 19 + TypeScript + Vite (UI)
- Rust (local system integration)
- Lucide icons

The UI and the Rust layer meet only at the Tauri command surface, which is
documented in [`docs/IPC_CONTRACT.md`](docs/IPC_CONTRACT.md).

## Run it

Prerequisites: Node.js 22 or later, npm, a Rust toolchain (`rustup`), and Xcode
Command Line Tools.

```bash
npm install
npm run tauri dev     # the desktop app — this is what you want
```

`npm run dev` serves the UI in a browser, but the Rust backend is absent there,
so the library and compatibility checks cannot work. The app says so plainly
rather than showing an empty library that looks real.

Checks:

```bash
npm run build                                              # tsc -b && vite build
cd src-tauri && cargo test --lib                           # unit tests
cd src-tauri && cargo clippy --all-targets -- -D warnings  # lints
```

## Initial test strategy

1. Confirm the library UI works.
2. Add a local game entry (persisted to disk, validated by the Rust layer).
3. Implement safe executable-path validation and native diagnostics. ✓
4. Select a small, legally obtained Windows game with minimal launcher/DRM dependencies.
5. Test startup, rendering, input, audio, exit behavior, and logs.
6. Publish the exact macOS version, hardware, compatibility-layer version, and outcome.

A game being listed in the UI does not mean it is compatible. Compatibility claims must be based on tests on real hardware.

## Safety and licensing

ReaPlay is for games the user has lawfully obtained. It will not bypass DRM, license checks, or store authentication. Compatibility components must come from official sources and be used according to their licenses.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)

## License

No license has been selected yet. Until one is added, all rights are reserved by the copyright holder.
