import { useState, useEffect } from 'react';
import { getTemplates, templateToFiles, buildAndPushProject } from '../../lib/api';
import { useQwen } from '../../hooks/useQwen';
import type { ProjectTemplate, GeneratedFile, QwenLocation } from '../../types';

interface Props {
  qwenLocation: QwenLocation | null;
  ghLoggedIn: boolean;
  onProjectCreated: () => void;
}

const SYSTEM_PROMPT = `You are an expert software engineer and code generator.
When asked to generate a project, respond ONLY with a valid JSON array. No preamble, no explanation, no markdown outside the array.
Format:
[
  {"path": "src/main.py", "content": "...full file content..."},
  {"path": "README.md", "content": "..."}
]
Rules:
- Always include a README.md
- Write complete, working, well-commented code
- Escape all special characters in content strings properly
- Do not truncate any file - write the full content
- Output ONLY the JSON array, nothing before or after it`;

type Step = 'setup' | 'preview' | 'building' | 'done';

export default function BuilderPanel({ qwenLocation, ghLoggedIn, onProjectCreated }: Props) {
  const [step, setStep] = useState<Step>('setup');
  const [templates, setTemplates] = useState<ProjectTemplate[]>([]);
  const [selectedTemplate, setSelectedTemplate] = useState<string>('');
  const [mode, setMode] = useState<'template' | 'freeform'>('template');
  const [projectName, setProjectName] = useState('');
  const [description, setDescription] = useState('');
  const [freeformPrompt, setFreeformPrompt] = useState('');
  const [isPrivate, setIsPrivate] = useState(false);
  const [outputDir, setOutputDir] = useState('C:\\Users');
  const [generatedFiles, setGeneratedFiles] = useState<GeneratedFile[]>([]);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [buildResult, setBuildResult] = useState<{ success: boolean; message: string; url?: string } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [generating, setGenerating] = useState(false);
  const [streamPreview, setStreamPreview] = useState('');

  const { generate } = useQwen();

  useEffect(() => {
    getTemplates().then(setTemplates).catch(console.error);
  }, []);

  const generateFiles = async () => {
    if (!projectName.trim()) { setError('Project name is required'); return; }
    setError(null);
    setGenerating(true);
    setStreamPreview('');

    try {
      let files: GeneratedFile[] = [];

      if (mode === 'template' && selectedTemplate) {
        files = await templateToFiles(selectedTemplate, projectName, description);
        // If we also have a freeform prompt AND qwen is available, enhance with AI
        if (freeformPrompt && qwenLocation?.found) {
          const prompt = `I have a ${selectedTemplate} project called "${projectName}".
${description}
Additional requirements: ${freeformPrompt}
Generate any EXTRA files needed beyond the base template. Return JSON array of {path, content}.`;
          const raw = await generate(prompt, SYSTEM_PROMPT, tok => setStreamPreview(p => p + tok));
          const extraFiles = parseFilesFromResponse(raw);
          files = mergFiles(files, extraFiles);
        }
      } else if (mode === 'freeform') {
        if (!qwenLocation?.found) {
          setError('Freeform generation requires Qwen. Please install via Ollama.');
          return;
        }
        const prompt = `Generate a complete software project called "${projectName}".
Description: ${description}
Requirements: ${freeformPrompt}
Return a JSON array of ALL project files with their full content.`;
        const raw = await generate(prompt, SYSTEM_PROMPT, tok => setStreamPreview(p => p + tok));
        files = parseFilesFromResponse(raw);
        if (!files.length) {
          setError('Qwen did not return valid files. Check the AI Chat tab and try again.');
          return;
        }
      }

      setGeneratedFiles(files);
      setSelectedFile(files[0]?.path ?? null);
      setStreamPreview('');
      setStep('preview');
    } catch (e: any) {
      setError(String(e));
    } finally {
      setGenerating(false);
    }
  };

  const buildProject = async () => {
    setStep('building');
    setError(null);
    try {
      const result = await buildAndPushProject({
        project_name: projectName,
        description,
        template_id: selectedTemplate || null,
        freeform_prompt: freeformPrompt || null,
        generated_files: generatedFiles,
        private_repo: isPrivate,
        output_dir: outputDir,
      });
      setBuildResult({
        success: result.success,
        message: result.message,
        url: result.repo_url ?? undefined,
      });
      setStep('done');
      if (result.success) onProjectCreated();
    } catch (e: any) {
      setError(String(e));
      setStep('preview');
    }
  };

  const reset = () => {
    setStep('setup');
    setProjectName('');
    setDescription('');
    setFreeformPrompt('');
    setGeneratedFiles([]);
    setSelectedFile(null);
    setBuildResult(null);
    setError(null);
    setSelectedTemplate('');
  };

  const selectedFileContent = generatedFiles.find(f => f.path === selectedFile)?.content ?? '';

  return (
    <div className="panel">
      <div className="panel-header">
        <h1>Project Builder</h1>
        <p>Generate a modular project with AI and push it to GitHub automatically.</p>
      </div>

      {error && <div className="error-banner">⚠️ {error}</div>}

      {/* ── Step 1: Setup ── */}
      {step === 'setup' && (
        <div className="builder-setup">
          <div className="form-group">
            <label>Project Name</label>
            <input
              className="input"
              placeholder="my-awesome-project"
              value={projectName}
              onChange={e => setProjectName(e.target.value.replace(/\s+/g, '-').toLowerCase())}
            />
          </div>

          <div className="form-group">
            <label>Description</label>
            <input
              className="input"
              placeholder="A brief description of what this project does"
              value={description}
              onChange={e => setDescription(e.target.value)}
            />
          </div>

          <div className="mode-toggle">
            <button className={`mode-btn ${mode === 'template' ? 'active' : ''}`} onClick={() => setMode('template')}>
              📋 From Template
            </button>
            <button className={`mode-btn ${mode === 'freeform' ? 'active' : ''}`} onClick={() => setMode('freeform')}>
              🤖 AI Freeform
            </button>
          </div>

          {mode === 'template' && (
            <div className="template-grid">
              {templates.map(t => (
                <div
                  key={t.id}
                  className={`template-card ${selectedTemplate === t.id ? 'selected' : ''}`}
                  onClick={() => setSelectedTemplate(t.id)}
                >
                  <div className="template-name">{t.name}</div>
                  <div className="template-lang">{t.language}</div>
                  <div className="template-desc">{t.description}</div>
                  <div className="template-tags">
                    {t.tags.map(tag => <span key={tag} className="tag">{tag}</span>)}
                  </div>
                </div>
              ))}
            </div>
          )}

          <div className="form-group">
            <label>
              {mode === 'freeform' ? 'Describe what you want built' : 'Additional AI requirements (optional)'}
            </label>
            <textarea
              className="textarea"
              rows={4}
              placeholder={
                mode === 'freeform'
                  ? 'e.g. A REST API with authentication, SQLite database, and CRUD endpoints for users and posts'
                  : 'e.g. Add a dark mode toggle, use Tailwind CSS, add Jest tests'
              }
              value={freeformPrompt}
              onChange={e => setFreeformPrompt(e.target.value)}
            />
          </div>

          <div className="form-row">
            <div className="form-group" style={{ flex: 1 }}>
              <label>Output Directory</label>
              <input
                className="input"
                value={outputDir}
                onChange={e => setOutputDir(e.target.value)}
                placeholder="C:\Users\Projects"
              />
            </div>
            <label className="checkbox-label">
              <input type="checkbox" checked={isPrivate} onChange={e => setIsPrivate(e.target.checked)} />
              Private repo
            </label>
          </div>

          {!ghLoggedIn && (
            <div className="warning-box">
              ⚠️ GitHub not connected. Project will be created locally only.
            </div>
          )}

          {generating && (
            <div className="stream-preview">
              <div className="stream-label">🤖 Qwen is generating...</div>
              <pre className="stream-text">{streamPreview || '...'}</pre>
            </div>
          )}

          <button
            className="btn btn-primary"
            onClick={generateFiles}
            disabled={generating || (!projectName) || (mode === 'template' && !selectedTemplate)}
          >
            {generating ? '⏳ Generating...' : '→ Generate Project'}
          </button>
        </div>
      )}

      {/* ── Step 2: Preview ── */}
      {step === 'preview' && (
        <div className="preview-panel">
          <div className="preview-header">
            <h2>Preview — {generatedFiles.length} files</h2>
            <div className="preview-actions">
              <button className="btn btn-secondary" onClick={() => setStep('setup')}>← Back</button>
              <button className="btn btn-primary" onClick={buildProject}>
                🚀 Build & Push to GitHub
              </button>
            </div>
          </div>

          <div className="preview-layout">
            <div className="file-tree">
              {generatedFiles.map(f => (
                <div
                  key={f.path}
                  className={`file-item ${selectedFile === f.path ? 'active' : ''}`}
                  onClick={() => setSelectedFile(f.path)}
                >
                  <span className="file-icon">{getFileIcon(f.path)}</span>
                  <span className="file-path">{f.path}</span>
                </div>
              ))}
            </div>
            <div className="file-content">
              {selectedFile && (
                <>
                  <div className="file-content-header">{selectedFile}</div>
                  <textarea
                    className="code-editor"
                    value={selectedFileContent}
                    onChange={e => setGeneratedFiles(files =>
                      files.map(f => f.path === selectedFile ? { ...f, content: e.target.value } : f)
                    )}
                  />
                </>
              )}
            </div>
          </div>
        </div>
      )}

      {/* ── Step 3: Building ── */}
      {step === 'building' && (
        <div className="building-state">
          <div className="spinner">⚙️</div>
          <h2>Building & Pushing...</h2>
          <p>Creating files, initializing git, creating GitHub repo, pushing...</p>
        </div>
      )}

      {/* ── Step 4: Done ── */}
      {step === 'done' && buildResult && (
        <div className="done-state">
          <div className="done-icon">{buildResult.success ? '✅' : '❌'}</div>
          <h2>{buildResult.success ? 'Project Created!' : 'Build Failed'}</h2>
          <p>{buildResult.message}</p>
          {buildResult.url && (
            <a href={buildResult.url} target="_blank" rel="noreferrer" className="repo-link">
              📁 {buildResult.url}
            </a>
          )}
          <button className="btn btn-primary" onClick={reset}>Build Another Project</button>
        </div>
      )}
    </div>
  );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function parseFilesFromResponse(raw: string): GeneratedFile[] {
  // Strategy 1: find a clean JSON array
  try {
    const cleaned = raw
      .replace(/```json\s*/gi, '')
      .replace(/```\s*/g, '')
      .trim();
    const start = cleaned.indexOf('[');
    const end = cleaned.lastIndexOf(']');
    if (start !== -1 && end !== -1 && end > start) {
      const json = cleaned.slice(start, end + 1);
      const parsed = JSON.parse(json);
      if (Array.isArray(parsed) && parsed.length > 0) {
        return parsed.filter(f => f.path && typeof f.content === 'string');
      }
    }
  } catch {}

  // Strategy 2: extract individual file objects even if array is malformed/truncated
  const files: GeneratedFile[] = [];
  const objRegex = /\{\s*"path"\s*:\s*"([^"]+)"\s*,\s*"content"\s*:\s*("(?:[^"\\]|\\.)*"|`[^`]*`)/g;
  let match;
  while ((match = objRegex.exec(raw)) !== null) {
    try {
      const path = match[1];
      // Parse the content string properly
      const contentRaw = match[2];
      const content = contentRaw.startsWith('`')
        ? contentRaw.slice(1, -1)
        : JSON.parse(contentRaw);
      if (path && typeof content === 'string') {
        files.push({ path, content });
      }
    } catch {}
  }
  if (files.length > 0) return files;

  // Strategy 3: look for markdown code blocks with file path comments
  const blockRegex = /#+\s*`?([^\n`]+\.\w+)`?\n```[\w]*\n([\s\S]*?)```/g;
  while ((match = blockRegex.exec(raw)) !== null) {
    files.push({ path: match[1].trim(), content: match[2] });
  }

  return files;
}

function mergFiles(base: GeneratedFile[], extra: GeneratedFile[]): GeneratedFile[] {
  const map = new Map(base.map(f => [f.path, f]));
  extra.forEach(f => map.set(f.path, f));
  return Array.from(map.values());
}

function getFileIcon(path: string): string {
  if (path.endsWith('.ts') || path.endsWith('.tsx')) return '📘';
  if (path.endsWith('.py')) return '🐍';
  if (path.endsWith('.rs')) return '🦀';
  if (path.endsWith('.json')) return '📋';
  if (path.endsWith('.md')) return '📝';
  if (path.endsWith('.css')) return '🎨';
  if (path.endsWith('.html')) return '🌐';
  return '📄';
}
