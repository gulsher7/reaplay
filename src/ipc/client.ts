/**
 * Typed wrappers over the Tauri command surface.
 *
 * Nothing else in the UI should call `invoke` directly: keeping it here is what
 * makes the command names and argument shapes checkable in one place.
 */

import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import { backendUnavailableError, toIpcError } from './errors';
import type {
  Diagnostics,
  GameEntry,
  ImportGameRequest,
  LaunchRecord,
  ProviderInfo,
} from './types';

/** Emitted by Rust when a launched game exits. */
const LAUNCH_FINISHED_EVENT = 'reaplay://launch-finished';

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

export const listProviders = (): Promise<ProviderInfo[]> =>
  call<ProviderInfo[]>('list_providers');

export const launchGame = (id: string): Promise<LaunchRecord> =>
  call<LaunchRecord>('launch_game', { id });

export const terminateLaunch = (launchId: string): Promise<void> =>
  call<void>('terminate_launch', { launchId });

export const listLaunches = (): Promise<LaunchRecord[]> =>
  call<LaunchRecord[]>('list_launches');

/**
 * Subscribe to launch completions. Resolves to an unsubscribe function, or to
 * a no-op when there is no backend to listen to.
 */
export async function onLaunchFinished(
  handler: (record: LaunchRecord) => void,
): Promise<() => void> {
  if (!backendAvailable()) return () => {};
  const unlisten = await listen<LaunchRecord>(LAUNCH_FINISHED_EVENT, (event) =>
    handler(event.payload),
  );
  return unlisten;
}

/** A redacted diagnostic report, safe to paste into a bug report. */
export const exportDiagnostics = (): Promise<string> => call<string>('export_diagnostics');
