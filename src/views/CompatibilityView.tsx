import { Check, Cpu, RefreshCw, X } from 'lucide-react';

import { ErrorNotice } from '../components/ErrorNotice';
import type { Diagnostics, IpcError } from '../ipc';

type Props = {
  diagnostics?: Diagnostics;
  status: 'loading' | 'ready' | 'error';
  error?: IpcError;
  onRefresh: () => void;
};

export function CompatibilityView({ diagnostics, status, error, onRefresh }: Props) {
  if (status === 'loading') {
    return <div className="skeleton-panel" aria-busy="true" aria-label="Detecting" />;
  }
  if (status === 'error' && error) {
    return <ErrorNotice error={error} onRetry={onRefresh} />;
  }
  if (!diagnostics) return null;

  return (
    <section>
      <div className="section-heading">
        <div>
          <h2>Compatibility</h2>
          <p>
            What ReaPlay found on this Mac. A detected tool is not a promise
            that any particular game will run.
          </p>
        </div>
        <button className="secondary-button" onClick={onRefresh}>
          <RefreshCw size={14} /> Re-check
        </button>
      </div>

      <dl className="host-facts">
        <div>
          <dt>macOS</dt>
          <dd>{diagnostics.osVersion}</dd>
        </div>
        <div>
          <dt>Architecture</dt>
          <dd>{diagnostics.arch}</dd>
        </div>
        <div>
          <dt>ReaPlay</dt>
          <dd>{diagnostics.appVersion}</dd>
        </div>
      </dl>

      <ul className="capability-list">
        {diagnostics.capabilities.map((capability) => (
          <li key={capability.id} className="capability-row">
            <span
              className={capability.available ? 'capability-dot yes' : 'capability-dot no'}
              aria-hidden="true"
            >
              {capability.available ? <Check size={13} /> : <X size={13} />}
            </span>
            <div className="capability-body">
              <strong>{capability.displayName}</strong>
              {capability.detail && <span>{capability.detail}</span>}
            </div>
            <span className={capability.available ? 'capability-tag yes' : 'capability-tag no'}>
              {capability.available ? 'Detected' : 'Not found'}
            </span>
          </li>
        ))}
      </ul>

      <p className="view-footnote">
        <Cpu size={13} aria-hidden="true" /> Detection is read-only. ReaPlay
        does not download, install or modify compatibility tools.
      </p>
    </section>
  );
}
