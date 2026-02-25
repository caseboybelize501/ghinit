import { useState } from 'react';
import { ghListPrs, ghListIssues, ghCreatePr, ghCreateIssue, gitCommitAndPush, ghBrowseRepo } from '../../lib/api';
import type { Repo, PullRequest, Issue, QwenLocation } from '../../types';

interface Props {
  repos: Repo[];
  loading: boolean;
  onRefresh: () => void;
  qwenLocation: QwenLocation | null;
}

type RepoTab = 'files' | 'prs' | 'issues' | 'commit';

export default function RepoPanel({ repos, loading, onRefresh, qwenLocation }: Props) {
  const [selectedRepo, setSelectedRepo] = useState<Repo | null>(null);
  const [repoTab, setRepoTab] = useState<RepoTab>('files');
  const [prs, setPrs] = useState<PullRequest[]>([]);
  const [issues, setIssues] = useState<Issue[]>([]);
  const [files, setFiles] = useState<any[]>([]);
  const [loadingData, setLoadingData] = useState(false);
  const [search, setSearch] = useState('');

  // Commit form
  const [commitMsg, setCommitMsg] = useState('');
  const [repoPath, setRepoPath] = useState('');
  const [commitResult, setCommitResult] = useState<string | null>(null);

  // PR form
  const [prTitle, setPrTitle] = useState('');
  const [prBody, setPrBody] = useState('');
  const [showPrForm, setShowPrForm] = useState(false);

  // Issue form
  const [issueTitle, setIssueTitle] = useState('');
  const [issueBody, setIssueBody] = useState('');
  const [showIssueForm, setShowIssueForm] = useState(false);

  const selectRepo = async (repo: Repo) => {
    setSelectedRepo(repo);
    setRepoTab('files');
    setLoadingData(true);
    try {
      const [f, p, i] = await Promise.all([
        ghBrowseRepo(repo.full_name),
        ghListPrs(repo.full_name),
        ghListIssues(repo.full_name),
      ]);
      setFiles(f);
      setPrs(p);
      setIssues(i);
    } catch (e) {
      console.error(e);
    } finally {
      setLoadingData(false);
    }
  };

  const handleCommit = async () => {
    if (!commitMsg || !repoPath) return;
    try {
      const result = await gitCommitAndPush(repoPath, commitMsg);
      setCommitResult(result.message);
      setCommitMsg('');
    } catch (e: any) {
      setCommitResult(`Error: ${e}`);
    }
  };

  const handleCreatePr = async () => {
    if (!selectedRepo || !prTitle) return;
    try {
      const pr = await ghCreatePr(selectedRepo.full_name, prTitle, prBody);
      setPrs(p => [pr, ...p]);
      setShowPrForm(false);
      setPrTitle(''); setPrBody('');
    } catch (e: any) {
      alert(`PR creation failed: ${e}`);
    }
  };

  const handleCreateIssue = async () => {
    if (!selectedRepo || !issueTitle) return;
    try {
      const issue = await ghCreateIssue(selectedRepo.full_name, issueTitle, issueBody);
      setIssues(i => [issue, ...i]);
      setShowIssueForm(false);
      setIssueTitle(''); setIssueBody('');
    } catch (e: any) {
      alert(`Issue creation failed: ${e}`);
    }
  };

  const filteredRepos = repos.filter(r =>
    r.name.toLowerCase().includes(search.toLowerCase()) ||
    r.description?.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="panel">
      <div className="panel-header">
        <h1>Repository Manager</h1>
        <button className="btn btn-secondary" onClick={onRefresh}>🔄 Refresh</button>
      </div>

      <div className="repo-layout">
        {/* Repo List */}
        <div className="repo-list">
          <input
            className="input"
            placeholder="🔍 Search repos..."
            value={search}
            onChange={e => setSearch(e.target.value)}
          />
          {loading && <div className="loading">Loading repos...</div>}
          {filteredRepos.map(repo => (
            <div
              key={repo.full_name}
              className={`repo-item ${selectedRepo?.full_name === repo.full_name ? 'active' : ''}`}
              onClick={() => selectRepo(repo)}
            >
              <div className="repo-item-name">
                {repo.private ? '🔒' : '📂'} {repo.name}
              </div>
              <div className="repo-item-meta">
                {repo.language && <span className="lang-badge">{repo.language}</span>}
                {repo.stars > 0 && <span>⭐ {repo.stars}</span>}
              </div>
              {repo.description && (
                <div className="repo-item-desc">{repo.description}</div>
              )}
            </div>
          ))}
        </div>

        {/* Repo Detail */}
        <div className="repo-detail">
          {!selectedRepo ? (
            <div className="empty-state">
              <div style={{ fontSize: 48 }}>📁</div>
              <p>Select a repository to view details</p>
            </div>
          ) : (
            <>
              <div className="repo-detail-header">
                <div>
                  <h2>{selectedRepo.name}</h2>
                  <a href={selectedRepo.url} target="_blank" rel="noreferrer" className="repo-link">
                    {selectedRepo.url}
                  </a>
                </div>
              </div>

              <div className="repo-tabs">
                {(['files', 'prs', 'issues', 'commit'] as RepoTab[]).map(t => (
                  <button
                    key={t}
                    className={`repo-tab ${repoTab === t ? 'active' : ''}`}
                    onClick={() => setRepoTab(t)}
                  >
                    {t === 'files' ? `📄 Files` : t === 'prs' ? `🔀 PRs (${prs.length})` : t === 'issues' ? `🐛 Issues (${issues.length})` : '📤 Commit'}
                  </button>
                ))}
              </div>

              {loadingData && <div className="loading">Loading...</div>}

              {/* Files */}
              {repoTab === 'files' && !loadingData && (
                <div className="file-list">
                  {files.map((f: any) => (
                    <div key={f.name} className="file-row">
                      <span>{f.type === 'dir' ? '📁' : '📄'} {f.name}</span>
                      <span className="file-size">{f.size ? `${(f.size / 1024).toFixed(1)}kb` : ''}</span>
                    </div>
                  ))}
                  {files.length === 0 && <p className="empty-text">No files found.</p>}
                </div>
              )}

              {/* PRs */}
              {repoTab === 'prs' && !loadingData && (
                <div>
                  <button className="btn btn-primary" style={{ marginBottom: 12 }} onClick={() => setShowPrForm(true)}>
                    + New Pull Request
                  </button>
                  {showPrForm && (
                    <div className="form-card">
                      <input className="input" placeholder="PR Title" value={prTitle} onChange={e => setPrTitle(e.target.value)} />
                      <textarea className="textarea" placeholder="Description (optional)" value={prBody} onChange={e => setPrBody(e.target.value)} rows={3} />
                      <div className="form-row">
                        <button className="btn btn-primary" onClick={handleCreatePr}>Create PR</button>
                        <button className="btn btn-secondary" onClick={() => setShowPrForm(false)}>Cancel</button>
                      </div>
                    </div>
                  )}
                  {prs.map(pr => (
                    <div key={pr.number} className="list-item">
                      <div className="list-item-title">#{pr.number} {pr.title}</div>
                      <div className="list-item-meta">
                        <span className={`state-badge ${pr.state}`}>{pr.state}</span>
                        <span>by {pr.author}</span>
                        <a href={pr.url} target="_blank" rel="noreferrer" className="link">View →</a>
                      </div>
                    </div>
                  ))}
                  {prs.length === 0 && <p className="empty-text">No open pull requests.</p>}
                </div>
              )}

              {/* Issues */}
              {repoTab === 'issues' && !loadingData && (
                <div>
                  <button className="btn btn-primary" style={{ marginBottom: 12 }} onClick={() => setShowIssueForm(true)}>
                    + New Issue
                  </button>
                  {showIssueForm && (
                    <div className="form-card">
                      <input className="input" placeholder="Issue Title" value={issueTitle} onChange={e => setIssueTitle(e.target.value)} />
                      <textarea className="textarea" placeholder="Description (optional)" value={issueBody} onChange={e => setIssueBody(e.target.value)} rows={3} />
                      <div className="form-row">
                        <button className="btn btn-primary" onClick={handleCreateIssue}>Create Issue</button>
                        <button className="btn btn-secondary" onClick={() => setShowIssueForm(false)}>Cancel</button>
                      </div>
                    </div>
                  )}
                  {issues.map(issue => (
                    <div key={issue.number} className="list-item">
                      <div className="list-item-title">#{issue.number} {issue.title}</div>
                      <div className="list-item-meta">
                        <span className={`state-badge ${issue.state}`}>{issue.state}</span>
                        <span>by {issue.author}</span>
                        {issue.labels.map(l => <span key={l} className="tag">{l}</span>)}
                        <a href={issue.url} target="_blank" rel="noreferrer" className="link">View →</a>
                      </div>
                    </div>
                  ))}
                  {issues.length === 0 && <p className="empty-text">No open issues.</p>}
                </div>
              )}

              {/* Commit */}
              {repoTab === 'commit' && (
                <div className="form-card">
                  <h3>Commit & Push</h3>
                  <div className="form-group">
                    <label>Local repo path</label>
                    <input
                      className="input"
                      placeholder="C:\Users\Projects\my-project"
                      value={repoPath}
                      onChange={e => setRepoPath(e.target.value)}
                    />
                  </div>
                  <div className="form-group">
                    <label>Commit message</label>
                    <input
                      className="input"
                      placeholder="feat: add new feature"
                      value={commitMsg}
                      onChange={e => setCommitMsg(e.target.value)}
                    />
                  </div>
                  <button className="btn btn-primary" onClick={handleCommit} disabled={!commitMsg || !repoPath}>
                    📤 Commit & Push
                  </button>
                  {commitResult && (
                    <div className="result-box">{commitResult}</div>
                  )}
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
