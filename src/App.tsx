import { useState } from 'react';

import { BackendBanner } from './components/BackendBanner';
import { Sidebar, type ViewId } from './components/Sidebar';
import { TopBar } from './components/TopBar';
import { useDiagnostics } from './hooks/useDiagnostics';
import { useLaunches } from './hooks/useLaunches';
import { useLibrary } from './hooks/useLibrary';
import { useProviders } from './hooks/useProviders';
import { backendAvailable } from './ipc';
import { CompatibilityView } from './views/CompatibilityView';
import { LaunchesView } from './views/LaunchesView';
import { LibraryView } from './views/LibraryView';
import { PlannedView } from './views/PlannedView';
import { SettingsView } from './views/SettingsView';

const SECTION_TITLE: Record<ViewId, string> = {
  library: 'Library',
  compatibility: 'Compatibility',
  environments: 'Environments',
  launches: 'Launch logs',
  settings: 'Settings',
};

export default function App() {
  const [view, setView] = useState<ViewId>('library');
  const backend = backendAvailable();
  const library = useLibrary();
  const diagnostics = useDiagnostics();
  const providers = useProviders();
  const launches = useLaunches();

  // Without a Tauri host the commands genuinely cannot run. The banner says so
  // once, so each view can render its honest empty state instead of an error
  // that reads like a bug.
  const libraryStatus = backend ? library.status : 'ready';
  const diagnosticsStatus = backend ? diagnostics.status : 'ready';
  const launchesStatus = backend ? launches.status : 'ready';

  const hostSummary =
    backend && diagnostics.diagnostics
      ? `macOS ${diagnostics.diagnostics.osVersion} · ${diagnostics.diagnostics.arch}`
      : null;

  const launchBlockedReason = providers.providers.find((p) => !p.available)?.detail;

  return (
    <div className="app-shell">
      <Sidebar
        active={view}
        onNavigate={setView}
        gameCount={backend && library.status === 'ready' ? library.games.length : null}
      />

      <main className="main-content">
        <TopBar section={SECTION_TITLE[view]} hostSummary={hostSummary} />

        <div className="page-content">
          {!backend && <BackendBanner />}

          {view === 'library' && (
            <LibraryView
              games={backend ? library.games : []}
              status={libraryStatus}
              error={library.error}
              backendAvailable={backend}
              onRetry={() => void library.refresh()}
              onImport={library.importGame}
              onRemove={library.removeGame}
              onLaunch={launches.launch}
              canLaunch={backend && providers.anyAvailable}
              {...(launchBlockedReason ? { launchBlockedReason } : {})}
            />
          )}

          {view === 'compatibility' &&
            (backend ? (
              <CompatibilityView
                diagnostics={diagnostics.diagnostics}
                status={diagnosticsStatus}
                error={diagnostics.error}
                onRefresh={() => {
                  void diagnostics.refresh();
                  void providers.refresh();
                }}
              />
            ) : (
              <PlannedView
                title="Compatibility detection needs the desktop app"
                description="Host and tool detection runs in the Rust backend. Start the desktop app to see what this Mac has."
                phase="DESKTOP ONLY"
              />
            ))}

          {view === 'environments' && (
            <PlannedView
              title="Environments"
              description="Per-game compatibility profiles and environment management are not built yet. Provider detection lives under Compatibility."
              phase="PHASE 4"
            />
          )}

          {view === 'launches' &&
            (backend ? (
              <LaunchesView
                launches={launches.launches}
                status={launchesStatus}
                error={launches.error}
                onRefresh={() => void launches.refresh()}
                onTerminate={launches.terminate}
              />
            ) : (
              <PlannedView
                title="Launch logs need the desktop app"
                description="Launching and process logging run in the Rust backend."
                phase="DESKTOP ONLY"
              />
            ))}

          {view === 'settings' && (
            <SettingsView
              diagnostics={diagnostics.diagnostics}
              status={diagnosticsStatus}
              error={diagnostics.error}
              onRefresh={() => void diagnostics.refresh()}
            />
          )}
        </div>
      </main>
    </div>
  );
}
