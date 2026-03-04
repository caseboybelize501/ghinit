import { useState, useEffect } from 'react';
import {
    listTrackedProjects, getEnvironmentInfo,
    openInVsCode, openProjectFolder, openInTerminal, openInCursor, copyToClipboard, runAntigravity,
    executeTask
} from '../../lib/api';
import type { ProjectRecord } from '../../types';

interface Props {
    activeProjectPath: string | null;
    setActiveProjectPath: (path: string | null) => void;
}

export default function DashboardPanel({ activeProjectPath, setActiveProjectPath }: Props) {
    const [projects, setProjects] = useState<ProjectRecord[]>([]);
    const [envInfo, setEnvInfo] = useState<string>('Loading...');
    const [loading, setLoading] = useState(true);
    const [copiedId, setCopiedId] = useState<string | null>(null);

    useEffect(() => {
        refresh();
    }, []);

    const refresh = () => {
        setLoading(true);
        Promise.all([
            listTrackedProjects(),
            getEnvironmentInfo()
        ]).then(([p, e]) => {
            setProjects(p);
            setEnvInfo(e);
        }).catch(err => console.error("Refresh failed:", err))
            .finally(() => setLoading(false));
    };

    return (
        <div className="panel dashboard-panel">
            <header className="panel-header">
                <div>
                    <h1>Dashboard</h1>
                    <p>Manage your active PurposeForge projects</p>
                </div>
                <div className="dashboard-actions" style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                    <button className="btn btn-secondary" onClick={refresh}>🔄 Refresh</button>
                    <div className="env-badge">{envInfo}</div>
                </div>
            </header>

            <section className="dashboard-stats">
                <div className="stat-card">
                    <span className="stat-value">{projects.length}</span>
                    <span className="stat-label">Tracked Projects</span>
                </div>
                <div className="stat-card">
                    <span className="stat-value">Active</span>
                    <span className="stat-label">System Engine</span>
                </div>
            </section>


            <section className="project-grid">
                <h2>Your Projects</h2>
                {loading ? (
                    <p>Loading projects...</p>
                ) : projects.length === 0 ? (
                    <div className="empty-state">
                        <p>No projects tracked yet. Build something amazing!</p>
                    </div>
                ) : (
                    <div className="grid">
                        {projects.map(p => (
                            <div key={p.id} className="project-card">
                                <div className="project-card-header" style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
                                    <h3>{p.name}</h3>
                                    <div className="project-quick-actions" style={{ display: 'flex', gap: '4px' }}>
                                        <button className="btn-icon" onClick={() => openInVsCode(p.path)} title="Open in VS Code">💻</button>
                                        <button className="btn-icon" onClick={() => openInCursor(p.path)} title="Open in Cursor">✨</button>
                                        <button className="btn-icon" onClick={() => openInTerminal(p.path)} title="Open Terminal">🖳</button>
                                        <button className="btn-icon" onClick={() => openProjectFolder(p.path)} title="Open Folder">📁</button>
                                        <button className="btn-icon" onClick={() => runAntigravity(p.path)} title="Open inside Antigravity">🛸</button>
                                        <button className="btn-icon" onClick={() => {
                                            copyToClipboard(p.path).then(() => {
                                                setCopiedId(p.id);
                                                setTimeout(() => setCopiedId(null), 2000);
                                            });
                                        }} title="Copy Path for Antigravity">
                                            {copiedId === p.id ? '✅' : '📋'}
                                        </button>
                                    </div>
                                </div>
                                <p className="path" style={{ fontSize: '11px', color: 'var(--text-dim)', wordBreak: 'break-all', marginBottom: '12px' }}>{p.path}</p>

                                <div className="tech-stack" style={{ display: 'flex', gap: '4px', flexWrap: 'wrap', marginBottom: '16px' }}>
                                    {p.tech_stack.length > 0 ? (
                                        p.tech_stack.map(t => <span key={t} className="tag">{t}</span>)
                                    ) : (
                                        <span className="tag" style={{ opacity: 0.5 }}>no tags</span>
                                    )}
                                </div>
                                <div className="project-card-footer" style={{ display: 'flex', gap: '8px' }}>
                                    <button
                                        className={`btn ${activeProjectPath === p.path ? 'btn-secondary' : 'btn-outline'}`}
                                        style={{ flex: 1, padding: '6px', fontSize: '12px' }}
                                        onClick={() => setActiveProjectPath(activeProjectPath === p.path ? null : p.path)}
                                    >
                                        {activeProjectPath === p.path ? '🎯 Focused' : '🎯 Focus Qwen'}
                                    </button>
                                </div>
                                <div style={{ display: 'flex', gap: '4px', marginTop: '8px' }}>
                                    <button
                                        className="btn btn-outline"
                                        style={{ flex: 1, padding: '4px', fontSize: '11px', color: 'var(--text-muted)' }}
                                        onClick={() => {
                                            executeTask(p.path, 'npm run dev').catch(alert)
                                        }}
                                        title="Run npm run dev"
                                    >
                                        ▶️ Dev server
                                    </button>
                                    <button
                                        className="btn btn-outline"
                                        style={{ flex: 1, padding: '4px', fontSize: '11px', color: 'var(--text-muted)' }}
                                        onClick={() => {
                                            executeTask(p.path, 'docker-compose up -d').catch(alert)
                                        }}
                                        title="Docker Compose Up"
                                    >
                                        🐳 Compose Up
                                    </button>
                                </div>
                            </div>
                        ))}
                    </div>
                )}
            </section>
        </div>
    );
}
