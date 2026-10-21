import { useMemo, useState } from 'react';
import {
  Activity,
  ArrowDownToLine,
  ChevronDown,
  CircleHelp,
  Gamepad2,
  Grid2X2,
  HardDrive,
  Library,
  Play,
  Plus,
  Search,
  Settings2,
  ShieldCheck,
  SlidersHorizontal,
  Sparkles,
  TerminalSquare,
} from 'lucide-react';

type Game = {
  id: string;
  title: string;
  subtitle: string;
  category: string;
  status: 'Ready to import' | 'Prototype';
  initials: string;
  accent: string;
};

const initialGames: Game[] = [
  {
    id: 'test-game',
    title: 'First Test Game',
    subtitle: 'Your first compatibility test',
    category: 'Windows game',
    status: 'Ready to import',
    initials: '01',
    accent: 'violet',
  },
  {
    id: 'local-library',
    title: 'Your game library',
    subtitle: 'Add a local game executable to begin',
    category: 'Library',
    status: 'Prototype',
    initials: 'R',
    accent: 'blue',
  },
];

function App() {
  const [activeNav, setActiveNav] = useState('Library');
  const [query, setQuery] = useState('');
  const [games, setGames] = useState(initialGames);
  const [selectedGame, setSelectedGame] = useState<Game>(initialGames[0]);
  const [showAdd, setShowAdd] = useState(false);
  const [gamePath, setGamePath] = useState('');
  const [gameTitle, setGameTitle] = useState('');

  const filteredGames = useMemo(
    () => games.filter((game) => game.title.toLowerCase().includes(query.toLowerCase())),
    [games, query],
  );

  const addGame = () => {
    const title = gameTitle.trim();
    const path = gamePath.trim();
    if (!title || !path) return;
    const game: Game = {
      id: crypto.randomUUID(),
      title,
      subtitle: path,
      category: 'Local executable',
      status: 'Ready to import',
      initials: title.slice(0, 2).toUpperCase(),
      accent: 'blue',
    };
    setGames((current) => [...current, game]);
    setSelectedGame(game);
    setGameTitle('');
    setGamePath('');
    setShowAdd(false);
  };

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark"><Gamepad2 size={21} strokeWidth={2.3} /></div>
          <div><span className="brand-name">ReaPlay</span><span className="brand-caption">GAME SPACE</span></div>
        </div>

        <div className="sidebar-label">WORKSPACE</div>
        <nav className="nav-list">
          <button className={activeNav === 'Library' ? 'nav-item active' : 'nav-item'} onClick={() => setActiveNav('Library')}><Library size={17} /> Library <span className="nav-count">{games.length}</span></button>
          <button className={activeNav === 'Compatibility' ? 'nav-item active' : 'nav-item'} onClick={() => setActiveNav('Compatibility')}><ShieldCheck size={17} /> Compatibility</button>
          <button className={activeNav === 'Diagnostics' ? 'nav-item active' : 'nav-item'} onClick={() => setActiveNav('Diagnostics')}><Activity size={17} /> Diagnostics</button>
        </nav>

        <div className="sidebar-label tools-label">TOOLS</div>
        <nav className="nav-list">
          <button className={activeNav === 'Environments' ? 'nav-item active' : 'nav-item'} onClick={() => setActiveNav('Environments')}><TerminalSquare size={17} /> Environments</button>
          <button className={activeNav === 'Settings' ? 'nav-item active' : 'nav-item'} onClick={() => setActiveNav('Settings')}><Settings2 size={17} /> Settings</button>
        </nav>

        <div className="sidebar-bottom">
          <div className="build-card">
            <div className="build-icon"><Sparkles size={16} /></div>
            <div><strong>Prototype build</strong><span>Phase 01 · Library UI</span></div>
            <span className="live-dot" />
          </div>
          <div className="profile"><div className="avatar">R</div><div className="profile-text"><strong>ReaPlay Lab</strong><span>Local workspace</span></div><ChevronDown size={15} /></div>
        </div>
      </aside>

      <main className="main-content">
        <header className="topbar">
          <div className="breadcrumb"><span>Workspace</span><span className="crumb-slash">/</span><strong>{activeNav}</strong></div>
          <div className="topbar-right"><span className="system-pill"><span className="system-dot" /> macOS · Apple Silicon</span><button className="icon-button" aria-label="Help"><CircleHelp size={17} /></button></div>
        </header>

        <div className="page-content">
          <section className="welcome-row">
            <div>
              <div className="eyebrow"><span className="eyebrow-line" /> EARLY ACCESS <span className="eyebrow-dot">·</span> EXPERIMENTAL</div>
              <h1>Your games.<br /><span>One place to play.</span></h1>
              <p className="intro">A small, focused launcher prototype for testing Windows game compatibility on your Mac.</p>
            </div>
            <div className="hero-orbit" aria-hidden="true"><div className="orbit orbit-one" /><div className="orbit orbit-two" /><div className="orbit-core"><Gamepad2 size={38} /></div><span className="orbit-star star-one">✳</span><span className="orbit-star star-two">✦</span></div>
          </section>

          <section className="status-strip">
            <div className="status-icon"><ShieldCheck size={18} /></div>
            <div className="status-copy"><strong>Prototype mode</strong><span>This UI does not launch games yet. We'll connect a real compatibility backend after the first UI test.</span></div>
            <span className="status-tag">UI ONLY</span>
          </section>

          {activeNav === 'Library' ? (
            <>
              <div className="section-heading">
                <div><h2>Game library <span className="heading-count">{games.length.toString().padStart(2, '0')}</span></h2><p>Manage your local games and test one at a time.</p></div>
                <button className="primary-button" onClick={() => setShowAdd(true)}><Plus size={16} /> Add a game</button>
              </div>

              <div className="library-toolbar">
                <div className="search-box"><Search size={16} /><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search your library..." aria-label="Search your library" /></div>
                <button className="filter-button" onClick={() => setQuery('')}><SlidersHorizontal size={15} /> Clear search</button>
                <div className="view-switch"><button className="view-active" aria-label="Grid view"><Grid2X2 size={16} /></button><button aria-label="List view" onClick={() => setActiveNav('Library')}><Library size={16} /></button></div>
              </div>

              <div className="game-grid">
                {filteredGames.map((game) => (
                  <button className={selectedGame.id === game.id ? 'game-card selected' : 'game-card'} key={game.id} onClick={() => setSelectedGame(game)}>
                    <div className={`game-art art-${game.accent}`}>
                      <div className="art-grid" />
                      <div className="art-number">{game.initials}</div>
                      <div className="art-logo"><Gamepad2 size={30} /></div>
                      <span className="art-label">REAPLAY LAB</span>
                      <span className="art-corner">EXPERIMENT 01</span>
                    </div>
                    <div className="game-card-info"><div><strong>{game.title}</strong><span>{game.subtitle}</span></div><span className="card-menu">•••</span></div>
                    <div className="game-card-footer"><span className="game-type"><HardDrive size={12} /> {game.category}</span><span className={game.status === 'Ready to import' ? 'game-state ready' : 'game-state'}><span /> {game.status}</span></div>
                  </button>
                ))}
                <button className="add-card" onClick={() => setShowAdd(true)}><span className="add-card-icon"><Plus size={21} /></span><strong>Add another game</strong><span>Import a local Windows executable</span></button>
              </div>

              <section className="selected-panel">
                <div className="selected-panel-main"><div className={`selected-icon art-${selectedGame.accent}`}><Gamepad2 size={22} /></div><div><div className="selected-kicker">SELECTED GAME</div><h3>{selectedGame.title}</h3><p>{selectedGame.subtitle}</p></div></div>
                <div className="selected-panel-actions"><button className="secondary-button" onClick={() => setShowAdd(true)}><ArrowDownToLine size={15} /> Configure import</button><button className="launch-button" onClick={() => window.alert('Game launching is not implemented yet. Next step: connect the macOS compatibility backend.')}><Play size={15} fill="currentColor" /> Launch <span className="launch-chevron">↗</span></button></div>
              </section>
            </>
          ) : (
            <section className="placeholder-page"><div className="placeholder-icon"><Settings2 size={23} /></div><h2>{activeNav}</h2><p>This section is planned for a later prototype milestone.</p><button className="secondary-button" onClick={() => setActiveNav('Library')}>Back to library</button></section>
          )}

          <footer className="page-footer"><span>REAPLAY <span className="footer-dot">●</span> EXPERIMENTAL SOFTWARE</span><span>Built for exploration, not a compatibility guarantee.</span></footer>
        </div>
      </main>

      {showAdd && <div className="modal-backdrop" role="presentation" onClick={(event) => { if (event.target === event.currentTarget) setShowAdd(false); }}><section className="modal" role="dialog" aria-modal="true" aria-labelledby="modal-title"><button className="modal-close" onClick={() => setShowAdd(false)} aria-label="Close">×</button><div className="modal-icon"><Plus size={20} /></div><h2 id="modal-title">Add a local game</h2><p>For this prototype, enter a title and the path to a game executable. This only saves it in the current browser session; it does not install or launch anything.</p><label>Game title<input autoFocus value={gameTitle} onChange={(event) => setGameTitle(event.target.value)} placeholder="e.g. My Test Game" /></label><label>Executable path<input value={gamePath} onChange={(event) => setGamePath(event.target.value)} placeholder="/path/to/game.exe" /></label><div className="modal-note"><ShieldCheck size={15} /> Use games you lawfully own. DRM and account checks are not bypassed.</div><div className="modal-actions"><button className="secondary-button" onClick={() => setShowAdd(false)}>Cancel</button><button className="primary-button" disabled={!gameTitle.trim() || !gamePath.trim()} onClick={addGame}>Add to library <Plus size={15} /></button></div></section></div>}
    </div>
  );
}

export default App;
