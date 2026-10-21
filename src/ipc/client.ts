/**
 * Typed wrappers over the Tauri command surface.
 *
 * Nothing else in the UI should call `invoke` directly: keeping it here is what
 * makes the command names and argument shapes checkable in one place.
 */

import { invoke, isTauri } from '@tauri-apps/api/core';

import { backendUnavailableError, toIpcError } from './errors';
import type { Diagnostics, GameEntry, ImportGameRequest } from './types';

/** Whether a Tauri host is present. False when served in a plain browser. */
export function backendAvailable(): boolean {
  try {
    return isTauri();
  } catch {
    return false;
  }
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!backendAvailable()) throw backendUnavailableError();
  try {
    return await invoke<T>(command, args);
  } catch (raw) {
    throw toIpcError(raw);
  }
}

export const listGames = (): Promise<GameEntry[]> => call<GameEntry[]>('list_games');

export const importGame = (request: ImportGameRequest): Promise<GameEntry> =>
  call<GameEntry>('import_game', { request });

export const removeGame = (id: string): Promise<void> => call<void>('remove_game', { id });

export const getDiagnostics = (): Promise<Diagnostics> => call<Diagnostics>('get_diagnostics');
