// ── Qwen ──────────────────────────────────────────────────────────────────────
export interface QwenLocation {
  found: boolean;
  method: 'ollama' | 'lmstudio' | 'binary' | 'ollama_no_model' | 'none';
  path: string | null;
  model: string | null;
}

// ── GitHub ────────────────────────────────────────────────────────────────────
export interface Repo {
  name: string;
  full_name: string;
  description: string | null;
  private: boolean;
  url: string;
  default_branch: string;
  stars: number;
  language: string | null;
}

export interface PullRequest {
  number: number;
  title: string;
  state: string;
  author: string;
  url: string;
  created_at: string;
}

export interface Issue {
  number: number;
  title: string;
  state: string;
  author: string;
  url: string;
  labels: string[];
}

export interface CommitResult {
  success: boolean;
  sha: string | null;
  message: string;
}

// ── Builder ───────────────────────────────────────────────────────────────────
export interface GeneratedFile {
  path: string;
  content: string;
}

export interface ProjectTemplate {
  id: string;
  name: string;
  description: string;
  language: string;
  tags: string[];
  structure: { path: string; content: string }[];
}

export interface Skill {
  id: string;
  name: string;
  description: string;
  prompt: string;
}

export interface BuildProjectRequest {
  project_name: string;
  description: string;
  template_id: string | null;
  freeform_prompt: string | null;
  generated_files: GeneratedFile[];
  private_repo: boolean;
  output_dir: string;
}

export interface BuildResult {
  success: boolean;
  local_path: string;
  repo_url: string | null;
  message: string;
}

// ── Project Tracker ───────────────────────────────────────────────────────────
export interface ProjectRecord {
  id: string;
  name: string;
  path: string;
  remote_url?: string;
  tech_stack: string[];
  last_modified: number;
}

// ── Module Injection ──────────────────────────────────────────────────────────
export interface ModuleDefinition {
  id: string;
  name: string;
  description: string;
  tech_stack: string[];
  files: { src: string; dest: string }[];
}

// ── App State ─────────────────────────────────────────────────────────────────
export type Tab = 'builder' | 'repos' | 'ai' | 'dashboard';

export interface AppState {
  qwen: QwenLocation | null;
  ghLoggedIn: boolean;
  ghUser: string | null;
  activeTab: Tab;
}
