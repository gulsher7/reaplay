import { useMemo, useState } from 'react';
import { FolderPlus, Play, Plus, Search } from 'lucide-react';

import { ErrorNotice } from '../components/ErrorNotice';
import { GameCard } from '../components/GameCard';
import { ImportGameModal } from '../components/ImportGameModal';
import type { GameEntry, ImportGameRequest, IpcError } from '../ipc';

type Props = {
  games: GameEntry[];
  status: 'loading' | 'ready' | 'error';
  error?: IpcError;
  backendAvailable: boolean;
  onRetry: () => void;
  onImport: (request: ImportGameRequest) => Promise<IpcError | null>;
  onRemove: (id: string) => Promise<IpcError | null>;
  onLaunch: (gameId: string) => Promise<IpcError | null>;
  /** False when no compatibility provider is installed. */
  canLaunch: boolean;
  /** Why launching is unavailable, when it is. */
  launchBlockedReason?: string;
};

export function LibraryView({
  games,
  status,
  error,
  backendAvailable,
  onRetry,
  onImport,
  onRemove,
  onLaunch,
  canLaunch,
  launchBlockedReason,
}: Props) {
  const [query, setQuery] = useState('');
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [showImport, setShowImport] = useState(false);
  const [removeError, setRemoveError] = useState<IpcError | null>(null);
  const [launchError, setLaunchError] = useState<IpcError | null>(null);
  const [launching, setLaunching] = useState(false);

  const filtered = useMemo(
    () => games.filter((game) => game.title.toLowerCase().includes(query.trim().toLowerCase())),
    [games, query],
  );
  const selected = games.find((game) => game.id === selectedId) ?? null;

  if (status === 'loading') {
    return (
      <section aria-busy="true" aria-label="Loading library">
        <div className="game-grid">
          {[0, 1, 2].map((key) => (
            <div key={key} className="skeleton-card" aria-hidden="true" />
          ))}
        </div>
      </section>
    );
  }

  if (status === 'error' && error) {
    return <ErrorNotice error={error} onRetry={onRetry} />;
  }

  return (
    <section>
      <div className="section-heading">
        <div>
          <h2>
            Game library{' '}
            <span className="heading-count">
              {games.length.toString().padStart(2, '0')}
            </span>
          </h2>
          <p>Local Windows executables you have added. Files stay where they are.</p>
        </div>
        <button
          className="primary-button"
          onClick={() => setShowImport(true)}
          disabled={!backendAvailable}
          title={backendAvailable ? undefined : 'Requires the desktop app'}
        >
          <Plus size={15} /> Add game
        </button>
      </div>

      {games.length > 0 && (
        <div className="library-toolbar">
          <div className="search-box">
            <Search size={14} aria-hidden="true" />
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search your library"
              aria-label="Search your library"
            />
          </div>
        </div>
      )}

      {removeError && <ErrorNotice error={removeError} />}
      {launchError && <ErrorNotice error={launchError} />}

      {games.length === 0 ? (
        <div className="empty-library">
          <div className="empty-icon" aria-hidden="true">
            <FolderPlus size={22} />
          </div>
          <h3>No games yet</h3>
          <p>
            Add a Windows executable to start a library. ReaPlay records the
            path and checks that the file is reachable — including on external
            drives like a Samsung T7.
          </p>
          <button
            className="primary-button"
            onClick={() => setShowImport(true)}
            disabled={!backendAvailable}
            title={backendAvailable ? undefined : 'Requires the desktop app'}
          >
            <Plus size={15} /> Add your first game
          </button>
        </div>
      ) : (
        <>
          <div className="game-grid">
            {filtered.map((entry) => (
              <GameCard
                key={entry.id}
                entry={entry}
                selected={entry.id === selectedId}
                onSelect={() => setSelectedId(entry.id)}
                onRemove={async () => {
                  const failure = await onRemove(entry.id);
                  setRemoveError(failure);
                  if (!failure && selectedId === entry.id) setSelectedId(null);
                }}
              />
            ))}
            <button
              className="add-card"
              onClick={() => setShowImport(true)}
              disabled={!backendAvailable}
            >
              <span className="add-card-icon" aria-hidden="true">
                <Plus size={18} />
              </span>
              <strong>Add a game</strong>
              <span>Point ReaPlay at an .exe</span>
            </button>
          </div>

          {filtered.length === 0 && (
            <p className="no-results">No games match “{query}”.</p>
          )}
        </>
      )}

      {selected && (
        <section className="selected-panel">
          <div className="selected-panel-main">
            <div className="selected-icon" aria-hidden="true">
              {selected.title.slice(0, 2).toUpperCase()}
            </div>
            <div>
              <span className="selected-kicker">SELECTED</span>
              <h3>{selected.title}</h3>
              <p>{selected.executablePath}</p>
            </div>
          </div>
          <div className="selected-panel-actions">
            <button className="secondary-button" disabled title="Planned for a later phase">
              Configure
            </button>
            <button
              className="launch-button"
              disabled={!canLaunch || selected.availability !== 'available' || launching}
              title={
                !canLaunch
                  ? launchBlockedReason ?? 'No compatibility provider is installed'
                  : selected.availability !== 'available'
                    ? 'This game\u2019s files are not reachable'
                    : undefined
              }
              onClick={async () => {
                setLaunching(true);
                setLaunchError(await onLaunch(selected.id));
                setLaunching(false);
              }}
            >
              <Play size={14} /> {launching ? 'Starting\u2026' : 'Launch'}
            </button>
          </div>
        </section>
      )}

      {showImport && (
        <ImportGameModal onClose={() => setShowImport(false)} onSubmit={onImport} />
      )}
    </section>
  );
}
