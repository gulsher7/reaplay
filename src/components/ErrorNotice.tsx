import { useState } from 'react';
import { AlertTriangle } from 'lucide-react';

import type { IpcError } from '../ipc';

type Props = {
  error: IpcError;
  onRetry?: () => void;
};

/**
 * Shows a backend error without hiding it. The stable `code` and any `detail`
 * stay reachable, because "something went wrong" is not a troubleshootable
 * message.
 */
export function ErrorNotice({ error, onRetry }: Props) {
  const [showDetail, setShowDetail] = useState(false);

  return (
    <section className="error-panel" role="alert">
      <div className="error-icon" aria-hidden="true">
        <AlertTriangle size={17} />
      </div>
      <div className="error-body">
        <strong>{error.message}</strong>
        <code className="error-code">{error.code}</code>
        {error.detail && (
          <>
            <button
              className="error-toggle"
              onClick={() => setShowDetail((shown) => !shown)}
              aria-expanded={showDetail}
            >
              {showDetail ? 'Hide details' : 'Show details'}
            </button>
            {showDetail && <pre className="error-detail">{error.detail}</pre>}
          </>
        )}
      </div>
      {onRetry && (
        <button className="secondary-button" onClick={onRetry}>
          Try again
        </button>
      )}
    </section>
  );
}
