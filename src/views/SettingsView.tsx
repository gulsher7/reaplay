import { ErrorNotice } from '../components/ErrorNotice';
import type { Diagnostics, IpcError } from '../ipc';

type Props = {
  diagnostics?: Diagnostics;
  status: 'loading' | 'ready' | 'error';
  error?: IpcError;
  onRefresh: () => void;
};

export function SettingsView({ diagnostics, status, error, onRefresh }: Props) {
  if (status === 'error' && error) {
    return <ErrorNotice error={error} onRetry={onRefresh} />;
  }

  return (
    <section>
      <div className="section-heading">
        <div>
          <h2>Settings</h2>
          <p>Where ReaPlay keeps your data.</p>
        </div>
      </div>

      <dl className="host-facts host-facts-wide">
        <div>
          <dt>Library file</dt>
          <dd className="mono">{diagnostics?.libraryPath ?? '—'}</dd>
        </div>
      </dl>

      <p className="view-footnote">
        Library metadata is written atomically, so an interrupted write leaves
        the previous file intact. A file that cannot be parsed is moved aside
        rather than deleted. No passwords, store credentials or authentication
        tokens are stored.
      </p>

      <div className="planned-block">
        <strong>Planned</strong>
        <span>
          Configurable library and environment locations are not implemented
          yet.
        </span>
      </div>
    </section>
  );
}
