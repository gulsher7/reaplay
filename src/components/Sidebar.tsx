import type { LucideIcon } from 'lucide-react';
import { Boxes, Cpu, Gamepad2, Library, ScrollText, Settings } from 'lucide-react';

export type ViewId = 'library' | 'compatibility' | 'environments' | 'launches' | 'settings';

type NavEntry = {
  id: ViewId;
  label: string;
  icon: LucideIcon;
  /** Nav items whose features are not built yet are marked, never faked. */
  planned?: boolean;
};

const PRIMARY: NavEntry[] = [
  { id: 'library', label: 'Library', icon: Library },
  { id: 'compatibility', label: 'Compatibility', icon: Cpu },
];

const SECONDARY: NavEntry[] = [
  { id: 'environments', label: 'Environments', icon: Boxes, planned: true },
  { id: 'launches', label: 'Launch logs', icon: ScrollText },
  { id: 'settings', label: 'Settings', icon: Settings },
];

type Props = {
  active: ViewId;
  onNavigate: (view: ViewId) => void;
  gameCount: number | null;
};

export function Sidebar({ active, onNavigate, gameCount }: Props) {
  const renderItem = (entry: NavEntry) => {
    const Icon = entry.icon;
    const isActive = active === entry.id;
    return (
      <button
        key={entry.id}
        className={isActive ? 'nav-item active' : 'nav-item'}
        onClick={() => onNavigate(entry.id)}
        aria-current={isActive ? 'page' : undefined}
      >
        <Icon size={17} aria-hidden="true" /> {entry.label}
        {entry.planned && <span className="nav-planned">Planned</span>}
        {entry.id === 'library' && gameCount !== null && (
          <span className="nav-count">{gameCount}</span>
        )}
      </button>
    );
  };

  return (
    <aside className="sidebar">
      <div className="brand">
        <div className="brand-mark" aria-hidden="true">
          <Gamepad2 size={20} />
        </div>
        <div>
          <strong className="brand-name">ReaPlay</strong>
          <span className="brand-caption">PROTOTYPE</span>
        </div>
      </div>

      <span className="sidebar-label">LIBRARY</span>
      <nav className="nav-list" aria-label="Main">
        {PRIMARY.map(renderItem)}
      </nav>

      <span className="sidebar-label tools-label">SYSTEM</span>
      <nav className="nav-list" aria-label="System">
        {SECONDARY.map(renderItem)}
      </nav>
    </aside>
  );
}
