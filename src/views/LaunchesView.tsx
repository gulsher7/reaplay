import { useState } from 'react';
import { RefreshCw, Square, Terminal } from 'lucide-react';

import { ErrorNotice } from '../components/ErrorNotice';
import type { IpcError, LaunchRecord, LaunchStatus } from '../ipc';

type Props = {
  launches: LaunchRecord[];
  status: 'loading' | 'ready' | 'error';
  error?: IpcError;
  onRefresh: () => void;
  onTerminate: (launchId: string) => Promise<IpcError | null>;
};

const STATUS_LABEL: Record<LaunchStatus, string> = {
  running: 'Running',
  exited: 'Exited',
  failed: 'Failed to start',
  terminated: 'Stopped',
};

const STATUS_CLASS: Record<LaunchStatus, string> = {
  running: 'launch-status running',
  exited: 'launch-status exited',
  failed: 'launch-status failed',
  terminated: 'launch-status stopped',
};

function formatTime(iso: string): string {
  const parsed = new Date(iso);
  return Number.isNaN(parsed.getTime()) ? iso : parsed.toLocaleString();
}

export function LaunchesView({ launches, status, error, onRefresh, onTerminate }: Props) {
  const [expanded, setExpanded] = useState<string | null>(null);
  const [actionError, setActionError] = useState<IpcError | null>(null);

  if (status === 'loading') {
    return <div className="skeleton-panel" aria-busy="true" aria-label="Loading launches" />;
  }
  if (status === 'error' && error) {
    return <ErrorNotice error={error} onRetry={onRefresh} />;
  }

  return (
    <section>
      <div className="section-heading">
        <div>
          <h2>Launch logs</h2>
          <p>
            Every launch ReaPlay attempted, newest first, with exactly what was
            executed and what came back.
          </p>
        </div>
        <button className="secondary-button" onClick={onRefresh}>
          <RefreshCw size={14} /> Refresh
        </button>
      </div>

      {actionError && <ErrorNotice error={actionError} />}

      {launches.length === 0 ? (
        <div className="empty-library">
          <div className="empty-icon" aria-hidden="true">
            <Terminal size={22} />
          </div>
          <h3>No launches yet</h3>
          <p>
            When you launch a game, what ran and how it exited is recorded here
            — including its output, so a failure can be diagnosed.
          </p>
        </div>
      ) : (
        <ul className="launch-list">
          {launches.map((record) => {
            const isOpen = expanded === record.id;
            return (
              <li key={record.id} className="launch-row">
                <div className="launch-head">
                  <div className="launch-main">
                    <strong>{record.gameTitle}</strong>
                    <span className="launch-meta">
                      {formatTime(record.startedAt)} · via {record.providerId}
                      {record.exitCode !== undefined && ` · exit ${record.exitCode}`}
                    </span>
                  </div>
                  <span className={STATUS_CLASS[record.status]}>
                    {STATUS_LABEL[record.status]}
                  </span>
                  {record.status === 'running' && (
                    <button
                      className="secondary-button"
                      onClick={async () => {
                        setActionError(await onTerminate(record.id));
                      }}
                    >
                      <Square size={12} /> Stop
                    </button>
                  )}
                  <button
                    className="error-toggle"
                    onClick={() => setExpanded(isOpen ? null : record.id)}
                    aria-expanded={isOpen}
                  >
                    {isOpen ? 'Hide' : 'Details'}
                  </button>
                </div>

                {isOpen && (
                  <div className="launch-detail">
                    <dl className="launch-facts">
                      <div>
                        <dt>Program</dt>
                        <dd className="mono">{record.program}</dd>
                      </div>
                      <div>
                        <dt>Arguments</dt>
                        <dd className="mono">
                          {record.args.length > 0
                            ? record.args.map((arg, index) => (
                                <code key={index} className="arg-chip">
                                  {arg}
                                </code>
                              ))
                            : '—'}
                        </dd>
                      </div>
                    </dl>
                    {record.error && <p className="launch-error">{record.error}</p>}
                    {record.stdoutTail && (
                      <>
                        <span className="launch-stream-label">stdout</span>
                        <pre className="error-detail">{record.stdoutTail}</pre>
                      </>
                    )}
                    {record.stderrTail && (
                      <>
                        <span className="launch-stream-label">stderr</span>
                        <pre className="error-detail">{record.stderrTail}</pre>
                      </>
                    )}
                    {!record.stdoutTail && !record.stderrTail && (
                      <p className="launch-no-output">This process produced no output.</p>
                    )}
                  </div>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}
