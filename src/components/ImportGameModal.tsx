import { useEffect, useRef, useState } from 'react';
import { Plus, ShieldCheck } from 'lucide-react';

import type { ImportGameRequest, IpcError } from '../ipc';
import { ErrorNotice } from './ErrorNotice';

type Props = {
  onClose: () => void;
  /** Resolves to an error to display inline, or `null` on success. */
  onSubmit: (request: ImportGameRequest) => Promise<IpcError | null>;
};

export function ImportGameModal({ onClose, onSubmit }: Props) {
  const [executablePath, setExecutablePath] = useState('');
  const [title, setTitle] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<IpcError | null>(null);
  const firstFieldRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    firstFieldRef.current?.focus();
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [onClose]);

  const canSubmit = executablePath.trim().length > 0 && !submitting;

  const submit = async () => {
    if (!canSubmit) return;
    setSubmitting(true);
    setError(null);
    const trimmedTitle = title.trim();
    const failure = await onSubmit({
      executablePath: executablePath.trim(),
      ...(trimmedTitle ? { title: trimmedTitle } : {}),
    });
    setSubmitting(false);
    if (failure) {
      setError(failure);
      return;
    }
    onClose();
  };

  return (
    <div
      className="modal-backdrop"
      role="presentation"
      onClick={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <section className="modal" role="dialog" aria-modal="true" aria-labelledby="import-title">
        <button className="modal-close" onClick={onClose} aria-label="Close">
          ×
        </button>
        <div className="modal-icon" aria-hidden="true">
          <Plus size={20} />
        </div>
        <h2 id="import-title">Add a local game</h2>
        <p>
          Point ReaPlay at a Windows executable you already have. The file is
          validated and recorded in your library — nothing is copied, installed
          or launched.
        </p>

        <form
          onSubmit={(event) => {
            event.preventDefault();
            void submit();
          }}
        >
          <label>
            Executable path
            <input
              ref={firstFieldRef}
              value={executablePath}
              onChange={(event) => setExecutablePath(event.target.value)}
              placeholder="/Volumes/T7/Games/MyGame/game.exe"
              spellCheck={false}
              autoComplete="off"
              required
            />
          </label>
          <label>
            Title <span className="label-hint">optional</span>
            <input
              value={title}
              onChange={(event) => setTitle(event.target.value)}
              placeholder="Derived from the file name if left blank"
              autoComplete="off"
            />
          </label>

          {error && <ErrorNotice error={error} />}

          <div className="modal-note">
            <ShieldCheck size={15} /> Use games you lawfully own. ReaPlay does
            not bypass DRM, licence or account checks.
          </div>
          <div className="modal-actions">
            <button type="button" className="secondary-button" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="primary-button" disabled={!canSubmit}>
              {submitting ? 'Validating…' : 'Add to library'} <Plus size={15} />
            </button>
          </div>
        </form>
      </section>
    </div>
  );
}
