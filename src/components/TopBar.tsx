import { Circle } from 'lucide-react';

type Props = {
  section: string;
  hostSummary: string | null;
};

export function TopBar({ section, hostSummary }: Props) {
  return (
    <header className="topbar">
      <div className="breadcrumb">
        <span>ReaPlay</span>
        <span className="crumb-slash" aria-hidden="true">
          /
        </span>
        <strong>{section}</strong>
      </div>
      {hostSummary && (
        <div className="topbar-right">
          <span className="system-pill">
            <Circle size={6} className="system-dot" aria-hidden="true" />
            {hostSummary}
          </span>
        </div>
      )}
    </header>
  );
}
