import { invoke } from '@tauri-apps/api/core';
import type {
  QwenLocation, Repo, PullRequest, Issue, CommitResult,
  GeneratedFile, ProjectTemplate, BuildProjectRequest, BuildResult
} from '../types';

// ── Qwen ──────────────────────────────────────────────────────────────────────
export const locateQwen = () =>
  invoke<QwenLocation>('locate_qwen');

export const qwenGenerate = (location: QwenLocation, prompt: string, system?: string) =>
  invoke<string>('qwen_generate', { location, prompt, system: system ?? null });

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

export const buildAndPushProject = (req: BuildProjectRequest) =>
  invoke<BuildResult>('build_and_push_project', { req });

export const templateToFiles = (templateId: string, projectName: string, description: string) =>
  invoke<GeneratedFile[]>('template_to_files', { templateId, projectName, description });
