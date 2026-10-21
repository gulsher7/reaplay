import { useCallback, useEffect, useState } from 'react';

import {
  importGame as importGameCommand,
  listGames,
  removeGame as removeGameCommand,
} from '../ipc';
import type { GameEntry, ImportGameRequest, IpcError } from '../ipc';

type LoadStatus = 'loading' | 'ready' | 'error';

type Library = {
  games: GameEntry[];
  status: LoadStatus;
  /** Set only when `status` is `'error'`: loading the library itself failed. */
  error?: IpcError;
  refresh: () => Promise<void>;
  /** Resolves to `null` on success, or the error to show the user. */
  importGame: (request: ImportGameRequest) => Promise<IpcError | null>;
  removeGame: (id: string) => Promise<IpcError | null>;
};

/**
 * Owns the game library. Mutations go through the backend and the list is
 * re-read afterwards rather than patched locally, so availability stays
 * accurate — an external drive can vanish between any two calls.
 */
export function useLibrary(): Library {
  const [games, setGames] = useState<GameEntry[]>([]);
  const [status, setStatus] = useState<LoadStatus>('loading');
  const [error, setError] = useState<IpcError | undefined>(undefined);

  const refresh = useCallback(async () => {
    try {
      const next = await listGames();
      setGames(next);
      setError(undefined);
      setStatus('ready');
    } catch (raw) {
      setError(raw as IpcError);
      setStatus('error');
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const importGame = useCallback(
    async (request: ImportGameRequest): Promise<IpcError | null> => {
      try {
        await importGameCommand(request);
        await refresh();
        return null;
      } catch (raw) {
        return raw as IpcError;
      }
    },
    [refresh],
  );

  const removeGame = useCallback(
    async (id: string): Promise<IpcError | null> => {
      try {
        await removeGameCommand(id);
        await refresh();
        return null;
      } catch (raw) {
        return raw as IpcError;
      }
    },
    [refresh],
  );

  return { games, status, error, refresh, importGame, removeGame };
}
