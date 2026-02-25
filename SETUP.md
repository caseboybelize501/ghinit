# PurposeForge — Setup Guide

## What You Just Built

```
PurposeForge
├── 🏗️  Builder Panel     — Template + AI freeform project generation → auto push to GitHub
├── 📁  Repo Manager      — Browse, commit, manage PRs & issues across all your repos  
└── 🤖  AI Chat           — Talk to Qwen Coder about anything
```

---

## Prerequisites

### 1. GitHub CLI (`gh`)
Download from https://cli.github.com  
Then authenticate:
```bash
gh auth login
```

### 2. Qwen Coder via Ollama (recommended)
```bash
# Install Ollama: https://ollama.ai
ollama pull qwen2.5-coder:7b     # fast, good quality
# or
ollama pull qwen2.5-coder:32b    # best quality, needs 32GB RAM
```

The app will automatically detect which Qwen model you have installed.

**Alternative: LM Studio**  
- Download from https://lmstudio.ai  
- Load any `qwen2.5-coder` model  
- Start the local server (default port 1234)  
- The app will auto-detect it

---

## Running the App

```bash
# Copy the generated files into your existing project:
# src-tauri/src/commands/qwen.rs    → src-tauri/src/commands/qwen.rs
# src-tauri/src/commands/github.rs  → src-tauri/src/commands/github.rs
# src-tauri/src/commands/builder.rs → src-tauri/src/commands/builder.rs
# src-tauri/src/main.rs             → src-tauri/src/main.rs
# src-tauri/Cargo.toml              → src-tauri/Cargo.toml (merge deps)
# src/*                             → src/*

cargo tauri dev
```

---

## How It Works

### Builder Panel
1. Enter a project name and description
2. Pick a **template** (React/TS, Python CLI, FastAPI, Rust lib, Tauri app)
3. Optionally add freeform AI requirements — Qwen will generate extra files
4. OR switch to **Freeform mode**: describe your entire project in natural language
5. Preview and edit all generated files before pushing
6. Click **Build & Push to GitHub** — it will:
   - Write all files to disk
   - `git init` and make the initial commit
   - `gh repo create` and push

### Repo Manager
- Browse all your GitHub repos
- View file structure (via GitHub API)
- Create PRs and Issues directly
- Commit & push local changes by specifying your local repo path

### AI Chat
- Full streaming chat with Qwen
- Ask about architecture, debugging, code review, anything
- Responses stream in real-time token by token

---

## Architecture

```
Frontend (React/TS)
    ↕ tauri invoke()
Rust Backend
    ├── qwen.rs    → HTTP to Ollama (localhost:11434) or LM Studio (localhost:1234)
    ├── github.rs  → spawns `gh` CLI subprocess  
    └── builder.rs → file system ops + git + gh repo create
```

### Qwen Detection Priority
1. **Ollama** — checks `ollama list` for any qwen*code* model
2. **LM Studio** — polls `localhost:1234/v1/models` for qwen models
3. **Binary** — searches common install paths
4. Sidebar shows **Rescan** button if not found

---

## Adding More Templates

Edit `src-tauri/src/commands/builder.rs` → `get_templates()` function.  
Add a new `ProjectTemplate` with a unique `id` and `structure` files.

---

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `gh: command not found` | Install GitHub CLI from cli.github.com |
| Qwen shows 🔴 | Run `ollama serve` then click Rescan |
| `ollama_no_model` | Run `ollama pull qwen2.5-coder` |
| Build fails: Git error | Ensure `git` is in PATH |
| Repo push fails | Run `gh auth status` to check login |
