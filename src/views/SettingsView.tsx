import { useState } from 'react';
import { Check, ClipboardCopy, FileText } from 'lucide-react';

import { ErrorNotice } from '../components/ErrorNotice';
import { exportDiagnostics } from '../ipc';
import type { Diagnostics, IpcError } from '../ipc';

type Props = {
  diagnostics?: Diagnostics;
  status: 'loading' | 'ready' | 'error';
  error?: IpcError;
  onRefresh: () => void;
  backendAvailable: boolean;
};

export function SettingsView({
  diagnostics,
  status,
  error,
  onRefresh,
  backendAvailable,
}: Props) {
  const [report, setReport] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [reportError, setReportError] = useState<IpcError | null>(null);

  if (status === 'error' && error) {
    return <ErrorNotice error={error} onRetry={onRefresh} />;
  }

  const generate = async () => {
    setReportError(null);
    setCopied(false);
    try {
      setReport(await exportDiagnostics());
    } catch (raw) {
      setReportError(raw as IpcError);
    }
  };

  const copy = async () => {
    if (!report) return;
    try {
      await navigator.clipboard.writeText(report);
      setCopied(true);
    } catch {
      // Clipboard access can be refused; the textarea below is still
      // selectable, so this is not worth an error panel.
      setCopied(false);
    }
  };

  return (
    <section>
      <div className="section-heading">
        <div>
          <h2>Settings</h2>
          <p>Where ReaPlay keeps your data, and how to report a problem.</p>
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

      <div className="section-heading settings-subsection">
        <div>
          <h2>Diagnostic report</h2>
          <p>
            Host facts, detected tools and recent launches. Paths under your
            home folder are shown as <code>~</code>, and no credentials are
            included.
          </p>
        </div>
        <button
          className="secondary-button"
          onClick={() => void generate()}
          disabled={!backendAvailable}
          title={backendAvailable ? undefined : 'Requires the desktop app'}
        >
          <FileText size={14} /> Generate
        </button>
      </div>

      {reportError && <ErrorNotice error={reportError} />}

      {report && (
        <>
          <div className="report-actions">
            <button className="secondary-button" onClick={() => void copy()}>
              {copied ? <Check size={14} /> : <ClipboardCopy size={14} />}
              {copied ? 'Copied' : 'Copy'}
            </button>
          </div>
          <textarea
            className="report-body"
            readOnly
            value={report}
            aria-label="Diagnostic report"
            rows={18}
          />
        </>
      )}

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
