import type { IpcError } from './types';

/** Narrow an unknown thrown value to our error shape. */
export function isIpcError(value: unknown): value is IpcError {
  return (
    typeof value === 'object' &&
    value !== null &&
    'code' in value &&
    'message' in value &&
    typeof (value as IpcError).code === 'string' &&
    typeof (value as IpcError).message === 'string'
  );
}

/**
 * Normalise anything thrown by `invoke` into an `IpcError`.
 *
 * Commands reject with a serialised `IpcError`, but a panic or a transport
 * failure can surface as a bare string or an `Error`, and the UI should not
 * have to care which it got.
 */
export function toIpcError(raw: unknown): IpcError {
  if (isIpcError(raw)) return raw;
  if (typeof raw === 'string') {
    return { code: 'INTERNAL', message: 'Something went wrong inside ReaPlay.', detail: raw };
  }
  if (raw instanceof Error) {
    return { code: 'INTERNAL', message: 'Something went wrong inside ReaPlay.', detail: raw.message };
  }
  return {
    code: 'INTERNAL',
    message: 'Something went wrong inside ReaPlay.',
    detail: safeStringify(raw),
  };
}

export const backendUnavailableError = (): IpcError => ({
  code: 'BACKEND_UNAVAILABLE',
  message: 'The ReaPlay backend is not running.',
  detail:
    'This page is open in a browser. Game data and compatibility checks are only available in the desktop app.',
});

function safeStringify(value: unknown): string {
  try {
    return JSON.stringify(value) ?? String(value);
  } catch {
    return String(value);
  }
}
