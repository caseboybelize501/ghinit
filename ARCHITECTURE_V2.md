# PurposeForge: Extended Architecture

## Overview
PurposeForge is an AI-powered project builder and repository manager. This document outlines the architecture for the next phase of features: Project Tracking, Repo Modifier Engine, Module Injection System, Execution Engine, and UI Dashboard.

## 1. System Components

### 1.1 Project Tracking System
- **Purpose**: Maintain a persistent record of all projects generated or imported into PurposeForge.
- **Storage**: Local SQLite database via `tauri-plugin-sql` or simple JSON file storage in the application data directory.
- **Schema**:
    - `id`: UUID
    - `name`: string
    - `path`: local absolute path
    - `remote_url`: GitHub URL (optional)
    - `status`: (active, archived, template)
    - `last_built`: timestamp
    - `metadata`: JSON (technology stack, modules installed)

### 1.2 Repo Modifier Engine
- **Purpose**: Programmatically modify existing codebases without destroying existing logic.
- **Mechanism**: 
    - AI-guided "patching": Qwen analyzes code and provides diffs.
    - Template-based insertion: Using markers (e.g., `// [PF-INJECT]`) or AST manipulation.
- **Workflow**:
    1. Read target file.
    2. Contextualize with project metadata.
    3. Generate modification strategy via Python Sidecar (AI).
    4. Apply changes via Rust File I/O.
    5. Verify with Linter/Build check.

### 1.3 Module Injection System
- **Purpose**: A library of "standard" features that can be added to any project.
- **Module Structure**: 
    - `manifest.json`: Defines dependencies, target files, and injection points.
    - `assets/`: Files to be copied.
    - `scripts/`: Initialization logic.
- **Injection Logic**: Handles merging `package.json`, updating imports, and adding configuration files.

### 1.4 Execution Engine
- **Purpose**: Run projects directly from the PurposeForge UI.
- **Features**:
    - Command output streaming to UI.
    - Common task presets (build, test, dev, deploy).
    - Environment variable management.
- **Implementation**: Rust `Command` wrapping with `child_process` output captured via Tauri events.

### 1.5 UI Dashboard
- **Purpose**: Central hub for project management.
- **Views**:
    - **Global Overview**: Stats, recent activity, Qwen status.
    - **Project Cards**: Quick actions (Run, Edit, Push, Delete).
    - **System Health**: Backend status, Sidecar logs.

## 2. Updated Data Flow
1. **User Request** -> **UI Dashboard**
2. **Dashboard** -> **Tauri Command (Rust)**
3. **Rust** -> **ProjectTracker (SQLite)** -> **Execution Engine (Shell)**
4. **Rust** -> **Python Sidecar (AI Analysis)** -> **Repo Modifier (File I/O)**
5. **Real-time Feedback** -> **Dashboard (WebSockets/Tauri Events)**
