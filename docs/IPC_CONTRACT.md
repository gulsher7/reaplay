# IPC contract

The single boundary between the React UI and the Rust application layer. Both
sides build against this file rather than against each other, which is what
lets them be worked on in parallel.

> **Status: skeleton — not yet agreed.** Phase 1 commands are sketched below so
> the shape is visible. Nothing here is binding until the Rust and UI sides both
> sign off, and a change after that goes through a handoff note on both sides.

## Rules

- Every command returns a structured error, never a bare string
  (`docs/ARCHITECTURE.md`: "Commands that modify files or execute programs must
  return structured errors").
- The UI renders backend-reported capability and never infers support itself.
- Paths crossing this boundary are untrusted until the Rust side validates them.
- Launch arguments are argument arrays, never assembled command strings.
- Types are named identically on both sides, `camelCase` on the wire.

## Error shape

```ts
type IpcError = {
  code: string;      // stable, machine-readable, e.g. "PATH_NOT_FOUND"
  message: string;   // human-readable, safe to display
  detail?: string;   // optional diagnostic, may be verbose
};
```

## Phase 1 commands (proposed)

| Command | Args | Returns | Notes |
|---|---|---|---|
| `list_games` | — | `Game[]` | Reads local metadata store. |
| `import_game` | `{ executablePath: string }` | `Game` | Validates the path exists and is executable. |
| `remove_game` | `{ id: string }` | `void` | Metadata only; touches no game files. |
| `list_capabilities` | — | `Capability[]` | Detection only; must not modify the host. |
| `get_diagnostics` | — | `Diagnostics` | Excludes secrets and personal data by default. |

## Types (proposed)

```ts
type Game = {
  id: string;              // stable UUID
  title: string;
  executablePath: string;
  installDirectory?: string;
  providerId?: string;
  profileId?: string;
  createdAt: string;       // ISO 8601
  updatedAt: string;       // ISO 8601
};

type Capability = {
  id: string;
  displayName: string;
  available: boolean;
  version?: string;
  detail?: string;         // why unavailable, when it is
};

type Diagnostics = {
  appVersion: string;
  osVersion: string;
  arch: string;
  capabilities: Capability[];
};
```

## Open questions

- [ ] Where does the metadata store live, and who owns its schema migrations?
- [ ] Does `import_game` derive `title` from the filename, or does the UI supply it?
- [ ] Are capabilities cached, or re-detected on every call?
