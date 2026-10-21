# Architecture

## High-level design

```text
React UI
  ├─ Library and game details
  ├─ Import and launch controls
  └─ Diagnostics and settings
          │ Tauri commands/events
          ▼
Rust application layer
  ├─ Game library metadata
  ├─ Executable/path validation
  ├─ Capability and tool discovery
  ├─ Launch configuration
  └─ Structured logs and diagnostics
          │ provider interface
          ▼
Compatibility providers (future)
  ├─ Provider availability/version checks
  ├─ Environment configuration
  └─ Launch process and result reporting
```

## Boundaries

- The UI presents backend-reported capabilities; it must not infer support from a game title.
- Every filesystem path is untrusted input and must be validated before use.
- Launch arguments should be represented as argument arrays, not assembled into a shell command string.
- Commands that modify files or execute programs must return structured errors.
- Do not silently download, replace, or execute compatibility components.
- Never collect store credentials or attempt to bypass DRM or license checks.

## Proposed game record

A game record should be versioned and kept separate from provider-specific configuration:

- `id`: stable UUID
- `title`: display title
- `executablePath`: local executable path
- `installDirectory`: optional root path
- `providerId`: optional compatibility provider
- `profileId`: optional game-specific profile
- `createdAt`, `updatedAt`: timestamps

## Provider interface (conceptual)

A provider should expose capability discovery, validation, launch, and diagnostics. Provider-specific behavior must remain isolated so it can be implemented, tested, or disabled independently.

The first iteration should only implement detection and diagnostics. Actual game launching should be added after the app can validate paths, report requirements, and capture failures reliably.
