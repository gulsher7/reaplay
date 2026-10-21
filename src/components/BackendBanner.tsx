import { MonitorOff } from 'lucide-react';

/**
 * Shown when the page is open in a browser rather than the desktop app. The
 * library and compatibility checks genuinely cannot work here, and saying so
 * is better than rendering an empty library that looks real.
 */
export function BackendBanner() {
  return (
    <section className="status-strip status-strip-warning" role="status">
      <div className="status-icon status-icon-warning" aria-hidden="true">
        <MonitorOff size={17} />
      </div>
      <div className="status-copy">
        <strong>Running without the ReaPlay backend</strong>
        <span>
          This is the web preview. Your game library, path validation and
          compatibility detection only work in the desktop app — run{' '}
          <code>npm run tauri dev</code> to start it.
        </span>
      </div>
      <span className="status-tag">PREVIEW</span>
    </section>
  );
}
