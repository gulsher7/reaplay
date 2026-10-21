import { useCallback, useEffect, useState } from 'react';

import {
  launchGame as launchGameCommand,
  listLaunches,
  onLaunchFinished,
  terminateLaunch as terminateLaunchCommand,
} from '../ipc';
import type { IpcError, LaunchRecord } from '../ipc';

type Launches = {
  launches: LaunchRecord[];
  status: 'loading' | 'ready' | 'error';
  error?: IpcError;
  refresh: () => Promise<void>;
  /** Resolves to `null` on success, or the error to show the user. */
  launch: (gameId: string) => Promise<IpcError | null>;
  terminate: (launchId: string) => Promise<IpcError | null>;
};

/**
 * Launch history plus live updates. Rust owns the process lifecycle and emits
 * an event when a game exits; this hook re-reads rather than patching, so the
 * list always reflects what the backend actually recorded.
 */
export function useLaunches(): Launches {
  const [launches, setLaunches] = useState<LaunchRecord[]>([]);
  const [status, setStatus] = useState<'loading' | 'ready' | 'error'>('loading');
  const [error, setError] = useState<IpcError | undefined>(undefined);

  const refresh = useCallback(async () => {
    try {
      setLaunches(await listLaunches());
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

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    void onLaunchFinished(() => {
      void refresh();
    }).then((stop) => {
      // The component may have unmounted while the listener was registering.
      if (cancelled) stop();
      else unlisten = stop;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [refresh]);

  const launch = useCallback(
    async (gameId: string): Promise<IpcError | null> => {
      try {
        await launchGameCommand(gameId);
        await refresh();
        return null;
      } catch (raw) {
        return raw as IpcError;
      }
    },
    [refresh],
  );

  const terminate = useCallback(
    async (launchId: string): Promise<IpcError | null> => {
      try {
        await terminateLaunchCommand(launchId);
        await refresh();
        return null;
      } catch (raw) {
        return raw as IpcError;
      }
    },
    [refresh],
  );

  return { launches, status, error, refresh, launch, terminate };
}
