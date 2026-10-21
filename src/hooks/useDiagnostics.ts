import { useCallback, useEffect, useState } from 'react';

import { getDiagnostics } from '../ipc';
import type { Diagnostics, IpcError } from '../ipc';

type DiagnosticsState = {
  diagnostics?: Diagnostics;
  status: 'loading' | 'ready' | 'error';
  error?: IpcError;
  refresh: () => Promise<void>;
};

/** Host information and detected tools. Re-read on demand, never cached to disk. */
export function useDiagnostics(): DiagnosticsState {
  const [diagnostics, setDiagnostics] = useState<Diagnostics | undefined>(undefined);
  const [status, setStatus] = useState<'loading' | 'ready' | 'error'>('loading');
  const [error, setError] = useState<IpcError | undefined>(undefined);

  const refresh = useCallback(async () => {
    try {
      const next = await getDiagnostics();
      setDiagnostics(next);
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

  return { diagnostics, status, error, refresh };
}
