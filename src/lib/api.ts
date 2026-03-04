import { invoke } from '@tauri-apps/api/core';
import type {
  QwenLocation, Repo, PullRequest, Issue, CommitResult,
  GeneratedFile, ProjectTemplate, BuildProjectRequest, BuildResult,
  ProjectRecord, ModuleDefinition, Skill
} from '../types';

// ── Qwen ──────────────────────────────────────────────────────────────────────
export const locateQwen = () =>
  invoke<QwenLocation>('locate_qwen');

export const qwenGenerate = (location: QwenLocation, prompt: string, system?: string, projectPath?: string | null) =>
  invoke<string>('qwen_generate', { location, prompt, system: system ?? null, projectPath: projectPath ?? null });

// ── GitHub Auth ───────────────────────────────────────────────────────────────
export const ghAuthStatus = () =>
  invoke<{ loggedIn: boolean; user?: string; error?: string }>('gh_auth_status');

export const ghAuthLogin = () =>
  invoke<string>('gh_auth_login');

// ── Repos ─────────────────────────────────────────────────────────────────────
export const ghListRepos = (limit?: number) =>
  invoke<Repo[]>('gh_list_repos', { limit: limit ?? null });

export const ghCreateRepo = (name: string, description: string | null, priv: boolean, localPath: string) =>
  invoke<Repo>('gh_create_repo', { name, description, private: priv, localPath });

export const ghCloneRepo = (fullName: string, targetDir: string) =>
  invoke<string>('gh_clone_repo', { fullName, targetDir });

export const ghBrowseRepo = (fullName: string) =>
  invoke<any[]>('gh_browse_repo', { fullName });

export const gitCommitAndPush = (repoPath: string, message: string, files?: string[]) =>
  invoke<CommitResult>('git_commit_and_push', { repoPath, message, files: files ?? null });

// ── PRs ───────────────────────────────────────────────────────────────────────
export const ghListPrs = (fullName: string, state?: string) =>
  invoke<PullRequest[]>('gh_list_prs', { fullName, state: state ?? null });

export const ghCreatePr = (fullName: string, title: string, body?: string, base?: string, draft?: boolean) =>
  invoke<PullRequest>('gh_create_pr', { fullName, title, body: body ?? null, base: base ?? null, draft: draft ?? false });

// ── Issues ────────────────────────────────────────────────────────────────────
export const ghListIssues = (fullName: string, state?: string) =>
  invoke<Issue[]>('gh_list_issues', { fullName, state: state ?? null });

export const ghCreateIssue = (fullName: string, title: string, body?: string, labels?: string[]) =>
  invoke<Issue>('gh_create_issue', { fullName, title, body: body ?? null, labels: labels ?? null });

// ── Builder ───────────────────────────────────────────────────────────────────
export const getTemplates = () =>
  invoke<ProjectTemplate[]>('get_templates');

export const getSkills = () =>
  invoke<Skill[]>('get_skills');

export const buildAndPushProject = (req: BuildProjectRequest) =>
  invoke<BuildResult>('build_and_push_project', { req });

export const templateToFiles = (templateId: string, projectName: string, description: string) =>
  invoke<GeneratedFile[]>('template_to_files', { templateId, projectName, description });

// ── Tracker ───────────────────────────────────────────────────────────────────
export const trackProject = (project: ProjectRecord) =>
  invoke<void>('track_project', { project });

export const listTrackedProjects = () =>
  invoke<ProjectRecord[]>('list_tracked_projects');

export const updateProjectStatus = (id: string, status: string) =>
  invoke<void>('update_project_status', { id, status });

// ── Modifier ──────────────────────────────────────────────────────────────────
export const modifyFile = (path: string, targetContent: string, replacement: string) =>
  invoke<void>('modify_file', { path, targetContent, replacement });

export const applyDiff = (path: string, diff: string) =>
  invoke<void>('apply_diff', { path, diff });

export const readFileContent = (path: string) =>
  invoke<string>('read_file_content', { path });

// ── Injector ──────────────────────────────────────────────────────────────────
export const listAvailableModules = () =>
  invoke<ModuleDefinition[]>('list_available_modules');

export const injectModule = (projectPath: string, moduleId: string) =>
  invoke<void>('inject_module', { projectPath, moduleId });

// ── Executor ──────────────────────────────────────────────────────────────────
export const executeTask = (cwd: string, commandStr: string) =>
  invoke<string>('execute_task', { cwd, commandStr });

export const streamTask = (cwd: string, commandStr: string, taskId: string) =>
  invoke<void>('stream_task', { cwd, commandStr, taskId });

export const openProjectFolder = (path: string) =>
  invoke<void>('open_project_folder', { path });

export const openInVsCode = (path: string) =>
  invoke<void>('open_in_vscode', { path });

export const openInCursor = (path: string) =>
  invoke<void>('open_in_cursor', { path });

export const openInTerminal = (path: string) =>
  invoke<void>('open_in_terminal', { path });

export const runAntigravity = (path: string) =>
  invoke<void>('run_antigravity', { path });

export const copyToClipboard = (text: string) =>
  invoke<void>('copy_to_clipboard', { text });

export const getEnvironmentInfo = () =>
  invoke<string>('get_environment_info');
