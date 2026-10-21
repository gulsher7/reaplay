import { Gamepad2, HardDriveDownload, Trash2 } from 'lucide-react';

import type { Availability, GameEntry } from '../ipc';

type Props = {
  entry: GameEntry;
  selected: boolean;
  onSelect: () => void;
  onRemove: () => void;
};

const AVAILABILITY_LABEL: Record<Availability, string> = {
  available: 'Files found',
  executableMissing: 'File missing',
  volumeUnavailable: 'Drive not connected',
};

const AVAILABILITY_CLASS: Record<Availability, string> = {
  available: 'game-state ready',
  executableMissing: 'game-state missing',
  volumeUnavailable: 'game-state offline',
};

export function GameCard({ entry, selected, onSelect, onRemove }: Props) {
  const initials = entry.title.slice(0, 2).toUpperCase();

  return (
    <div className={selected ? 'game-card selected' : 'game-card'}>
      <button
        className="game-card-hit"
        onClick={onSelect}
        aria-pressed={selected}
        aria-label={`Select ${entry.title}`}
      >
        <div className="game-art art-violet">
          <div className="art-grid" aria-hidden="true" />
          <div className="art-logo" aria-hidden="true">
            <Gamepad2 size={26} />
          </div>
          <span className="art-label">{initials}</span>
        </div>
        <div className="game-card-info">
          <div>
            <strong title={entry.title}>{entry.title}</strong>
            <span title={entry.executablePath}>{entry.executablePath}</span>
          </div>
        </div>
        <div className="game-card-footer">
          <span className="game-type">
            <HardDriveDownload size={12} /> Local file
          </span>
          <span className={AVAILABILITY_CLASS[entry.availability]}>
            <span aria-hidden="true" />
            {AVAILABILITY_LABEL[entry.availability]}
          </span>
        </div>
      </button>
      <button
        className="card-remove"
        onClick={onRemove}
        aria-label={`Remove ${entry.title} from library`}
        title="Remove from library (does not delete the game files)"
      >
        <Trash2 size={14} />
      </button>
    </div>
  );
}
