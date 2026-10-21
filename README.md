# ReaPlay

**An experimental macOS launcher for Windows games on Apple Silicon.**

ReaPlay is an early-stage desktop app exploring how to manage game libraries, compatibility environments, and launch configuration for Windows games on macOS. It is not a compatibility guarantee: results depend on each game, macOS version, hardware, and available translation layers.

## Status

🚧 **Prototype / scaffold stage.** This repository currently establishes the product direction and architecture. The compatibility backend is not implemented yet.

## Goals

- A clean macOS-native-feeling game library and launcher.
- Per-game launch configuration and compatibility profiles.
- A modular backend for detecting installed tools and managing game environments.
- Diagnostics that make launch failures easier to understand.
- Clear reporting of requirements and known limitations.

## Planned stack

- **Tauri 2** for the desktop shell
- **React + TypeScript + Vite** for the UI
- **Rust** for local system integration
- A pluggable compatibility-provider layer (future work)

The frontend must never assume that a compatibility tool is installed or supported. Backend capability checks should be authoritative.

## Safety and scope

ReaPlay is intended to launch games the user has lawfully obtained and installed. It will not implement DRM, license, or account-authentication bypasses. Users remain responsible for complying with game-store terms and applicable licenses. Compatibility components must be obtained from their official sources and used under their respective licenses.

## Development

Prerequisites:

- Node.js (current LTS recommended)
- Rust toolchain
- Tauri prerequisites for macOS
- Xcode Command Line Tools

Once the initial Tauri app scaffold is generated, install dependencies and start its development server using the scripts defined in `package.json`.

## Initial roadmap

1. [x] Define product scope and modular architecture.
2. [ ] Add Tauri + React + TypeScript application scaffold.
3. [ ] Build the library and game-detail UI.
4. [ ] Add a Rust capability/diagnostics API.
5. [ ] Implement import of an existing local game executable.
6. [ ] Add compatibility-provider discovery and per-game profiles.
7. [ ] Add launch logs, validation, and diagnostics.
8. [ ] Test with legally obtained games and document compatibility results.

## License

No license has been selected yet. Until one is added, all rights are reserved by the copyright holder; do not assume the code is available for reuse.
