# PurposeForge: Core Skillset & Coding Laws

## 🏛️ System Architecture
[CORE] PurposeForge is a modular project builder consisting of:
1. **Frontend**: React + TypeScript + Vite.
2. **Backend**: Rust (Tauri) for system operations (File I/O, Git, Exec).
3. **AI Core**: Qwen-2.5-Coder (local via Ollama) for generation and analysis.
4. **Sidecar**: [PYTHON] Python-based AI utility engine for complex tasks.

## 📜 Coding Laws
- [TS] **Strict Typing**: Always use TypeScript interfaces. Never use `any`.
- [RUST] **Safety First**: Use Rust for all destructive file operations.
- [CORE] **AI Context**: Always wrap code suggestions in Markdown blocks.
- [TS] **Modularity**: Components must be reusable and reside in `src/components`.

## 🛠️ API Surface (Tauri Commands)
- `locate_qwen`: Finds local Ollama instance.
- `qwen_generate(prompt, system)`: Primary AI completion.
- `track_project(record)`: Persists project metadata.
- `list_tracked_projects()`: Retrieves project history.
- `execute_task(cwd, command)`: Runs shell commands in project context.

## 🧪 Few-Shot Modification Examples

### Task: Add a new API route
**Before:**
```python
from fastapi import FastAPI
app = FastAPI()
```
**After:**
```python
from fastapi import FastAPI
app = FastAPI()

@app.get("/health")
def health():
    return {"status": "ok"}
```

### Task: Inject PurposeForge Logic
**Target Code:**
```typescript
export function init() {
  console.log("Starting...");
}
```
**Injected Code:**
```typescript
import { trackProject } from './lib/api';
export function init() {
  console.log("Starting...");
  // [PF-INJECTION]
  trackProject({ ... });
}
```

### Task: Codebase Patching (Repo Modifier)
**Original File:**
```python
def process_data(data):
    return data.strip()
```
**Goal:** Add logging to the process function.
**AI Patch Instruction:**
Inject a logging statement before return.
**Patch Result:**
```python
import logging
def process_data(data):
    logging.info(f"Processing: {data}")
    return data.strip()
```
