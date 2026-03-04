import { useState, useEffect, useCallback } from 'react';
import { ghAuthStatus, ghListRepos } from './lib/api';
import { useQwen } from './hooks/useQwen';
import BuilderPanel from './components/builder/BuilderPanel';
import RepoPanel from './components/repo/RepoPanel';
import AIPanel from './components/ai/AIPanel';
import DashboardPanel from './components/dashboard/DashboardPanel';
import type { Tab, Repo } from './types';
import './App.css';

export default function App() {
  const [tab, setTab] = useState<Tab>('dashboard');
  const [ghLoggedIn, setGhLoggedIn] = useState(false);
  const [ghUser, setGhUser] = useState<string | null>(null);
  const [repos, setRepos] = useState<Repo[]>([]);
  const [reposLoading, setReposLoading] = useState(false);
  const [activeProjectPath, setActiveProjectPath] = useState<string | null>(null);
  const { location: qwenLocation, scanning: qwenScanning, scan: scanQwen } = useQwen();

  // Check GitHub auth on mount
  useEffect(() => {
    ghAuthStatus().then(status => {
      setGhLoggedIn(status.loggedIn);
      setGhUser(status.user ?? null);
      if (status.loggedIn) loadRepos();
    }).catch(console.error);
  }, []);

  const loadRepos = useCallback(async () => {
    setReposLoading(true);
    try {
      const r = await ghListRepos(50);
      setRepos(r);
    } catch (e) {
      console.error('Failed to load repos:', e);
    } finally {
      setReposLoading(false);
    }
  }, []);

  const handleGhLogin = async () => {
    // gh auth login must be run in a terminal — this just rechecks status
    try {
      const status = await ghAuthStatus();
      setGhLoggedIn(status.loggedIn);
      setGhUser(status.user ?? null);
      if (status.loggedIn) {
        loadRepos();
      } else {
        alert('Not authenticated. Run: gh auth login  in your terminal, then click Connect again.');
      }
    } catch (e) {
      alert('GitHub CLI not found. Install from cli.github.com then run: gh auth login');
    }
  };

  const qwenStatus = qwenScanning
    ? '⏳ Scanning...'
    : qwenLocation?.found
      ? `🟢 ${qwenLocation.model ?? 'Qwen'} (${qwenLocation.method})`
      : '🔴 Qwen not found';

  const ghStatus = ghLoggedIn
    ? `🟢 ${ghUser ?? 'GitHub'}`
    : '🔴 Not connected';

  return (
    <div className="app">
      {/* Sidebar */}
      <aside className="sidebar">
        <div className="sidebar-logo">
          <span className="logo-icon">⚒️</span>
          <span className="logo-text">PurposeForge</span>
        </div>

        <nav className="sidebar-nav">
          {(['dashboard', 'builder', 'repos', 'ai'] as Tab[]).map(t => (
            <button
              key={t}
              className={`nav-item ${tab === t ? 'active' : ''}`}
              onClick={() => setTab(t)}
            >
              <span className="nav-icon">
                {t === 'dashboard' ? '📊' : t === 'builder' ? '🏗️' : t === 'repos' ? '📁' : '🤖'}
              </span>
              <span>{t === 'dashboard' ? 'Dashboard' : t === 'builder' ? 'Builder' : t === 'repos' ? 'Repos' : 'AI Chat'}</span>
            </button>
          ))}
        </nav>

        {/* Status panel */}
        <div className="sidebar-status">
          <div className="status-row">
            <span className="status-label">Qwen</span>
            <span className="status-value">{qwenStatus}</span>
          </div>
          {!qwenLocation?.found && !qwenScanning && (
            <button className="status-action" onClick={scanQwen}>Rescan</button>
          )}

          <div className="status-row" style={{ marginTop: 12 }}>
            <span className="status-label">GitHub</span>
            <span className="status-value">{ghStatus}</span>
          </div>
          {!ghLoggedIn && (
            <button className="status-action" onClick={handleGhLogin}>Connect</button>
          )}
        </div>

        {/* Setup hint if missing deps */}
        {(!qwenLocation?.found || !ghLoggedIn) && (
          <div className="setup-hint">
            {!qwenLocation?.found && (
              <p>Install Qwen:<br />
                <code>ollama pull qwen2.5-coder</code>
              </p>
            )}
            {!ghLoggedIn && (
              <p>Install gh CLI from<br />
                <code>cli.github.com</code>
              </p>
            )}
          </div>
        )}
      </aside>

      {/* Main content */}
      <main className="main-content">
        {tab === 'dashboard' && (
          <DashboardPanel
            activeProjectPath={activeProjectPath}
            setActiveProjectPath={setActiveProjectPath}
          />
        )}
        {tab === 'builder' && (
          <BuilderPanel
            qwenLocation={qwenLocation}
            ghLoggedIn={ghLoggedIn}
            onProjectCreated={loadRepos}
            activeProjectPath={activeProjectPath}
          />
        )}
        {tab === 'repos' && (
          <RepoPanel
            repos={repos}
            loading={reposLoading}
            onRefresh={loadRepos}
          />
        )}
        {tab === 'ai' && (
          <AIPanel
            qwenLocation={qwenLocation}
            activeProjectPath={activeProjectPath}
          />
        )}
      </main>
    </div>
  );
}
