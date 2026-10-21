/**
 * TypeScript mirror of the Rust models in `src-tauri/src/model.rs`.
 *
 * These two files are the contract described in `docs/IPC_CONTRACT.md`. When
 * one changes, the other has to change with it — there is no code generation
 * keeping them in step.
 */

/** Stable error codes from `src-tauri/src/error.rs`, plus frontend-only ones. */
export type IpcErrorCode =
  | 'PATH_NOT_ABSOLUTE'
  | 'PATH_NOT_FOUND'
  | 'PATH_NOT_A_FILE'
  | 'PATH_INVALID'
  | 'PATH_UNREADABLE'
  | 'VOLUME_UNAVAILABLE'
  | 'GAME_NOT_FOUND'
  | 'GAME_ALREADY_IMPORTED'
  | 'STORE_UNREADABLE'
  | 'STORE_UNWRITABLE'
  | 'STORE_CORRUPT'
  | 'INTERNAL'
  /** Raised by the client, not the backend: there is no Tauri host present. */
  | 'BACKEND_UNAVAILABLE';

export type IpcError = {
  code: IpcErrorCode | string;
  message: string;
  detail?: string;
};

export type Game = {
  id: string;
  title: string;
  executablePath: string;
  installDirectory?: string;
  providerId?: string;
  profileId?: string;
  /** RFC 3339. */
  createdAt: string;
  updatedAt: string;
};

/** Whether a game's files are reachable right now, re-checked on every read. */
export type Availability = 'available' | 'executableMissing' | 'volumeUnavailable';

export type GameEntry = Game & { availability: Availability };

export type Capability = {
  id: string;
  displayName: string;
  available: boolean;
  version?: string;
  /** Why it is unavailable, when it is. */
  detail?: string;
};

export type Diagnostics = {
  appVersion: string;
  osVersion: string;
  arch: string;
  capabilities: Capability[];
  libraryPath: string;
};

export type ImportGameRequest = {
  executablePath: string;
  /** Omit to let the backend derive one from the file name. */
  title?: string;
};
