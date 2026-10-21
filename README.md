# ReaPlay

**An experimental macOS launcher for Windows games on Apple Silicon.**

The project will start with a small, runnable UI prototype and one carefully chosen test game. We will validate each layer before attempting larger or more demanding titles. ReaPlay is independent of Highball; it will not depend on Highball being installed.

## Current status

- [x] Product scope and architecture notes
- [x] Initial interactive React + TypeScript UI prototype
- [ ] Run the UI locally and refine the flow
- [ ] Add Tauri 2 and the native Rust backend
- [ ] Detect host capabilities and installed compatibility tools
- [ ] Import a lawfully obtained local game
- [ ] Test a small game and record reproducible results
- [ ] Expand compatibility only after the first test works

**Important:** the current UI is a visual prototype. It does not install, configure, or launch Windows games yet.

## Prototype stack

- React 19
- TypeScript
- Vite
- Lucide icons

Tauri and Rust will be added after the UI flow is agreed, so we can separate UI iteration from platform integration.

## Run the prototype

Prerequisites: Node.js 22 or later and npm.

```bash
npm install
npm run dev
```

Open the local URL printed by Vite. To check the production build:

```bash
npm run build
```

## Initial test strategy

1. Confirm the library UI works.
2. Add a local game entry (currently session-only demo data).
3. Implement safe executable-path validation and native diagnostics.
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
