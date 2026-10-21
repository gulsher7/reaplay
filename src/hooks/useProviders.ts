import { useCallback, useEffect, useState } from 'react';

import { listProviders } from '../ipc';
import type { IpcError, ProviderInfo } from '../ipc';

type Providers = {
  providers: ProviderInfo[];
  /** True when at least one provider could actually run a game. */
  anyAvailable: boolean;
  status: 'loading' | 'ready' | 'error';
  error?: IpcError;
  refresh: () => Promise<void>;
};

export function useProviders(): Providers {
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [status, setStatus] = useState<'loading' | 'ready' | 'error'>('loading');
  const [error, setError] = useState<IpcError | undefined>(undefined);

  const refresh = useCallback(async () => {
    try {
      setProviders(await listProviders());
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

  return {
    providers,
    anyAvailable: providers.some((provider) => provider.available),
    status,
    error,
    refresh,
  };
}
