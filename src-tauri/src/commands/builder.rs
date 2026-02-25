use std::fs;
use std::process::Command;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectModule {
    pub name: String,
    pub description: String,
    pub files: Vec<GeneratedFile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneratedFile {
    pub path: String,    // relative path within project, e.g. "src/main.py"
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub language: String,
    pub tags: Vec<String>,
    pub structure: Vec<TemplateFile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateFile {
    pub path: String,
    pub content: String, // may contain {{placeholders}}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildProjectRequest {
    pub project_name: String,
    pub description: String,
    pub template_id: Option<String>,
    pub freeform_prompt: Option<String>,
    pub generated_files: Vec<GeneratedFile>, // filled by AI before calling this
    pub private_repo: bool,
    pub output_dir: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildResult {
    pub success: bool,
    pub local_path: String,
    pub repo_url: Option<String>,
    pub message: String,
}

/// Returns the built-in template library
#[tauri::command]
pub async fn get_templates() -> Result<Vec<ProjectTemplate>, String> {
    Ok(vec![
        ProjectTemplate {
            id: "react-ts".into(),
            name: "React + TypeScript".into(),
            description: "Vite-powered React app with TypeScript".into(),
            language: "TypeScript".into(),
            tags: vec!["frontend".into(), "react".into(), "vite".into()],
            structure: vec![
                TemplateFile { path: "src/App.tsx".into(), content: "// {{description}}\nexport default function App() {\n  return <div>{{project_name}}</div>;\n}\n".into() },
                TemplateFile { path: "src/main.tsx".into(), content: "import React from 'react';\nimport ReactDOM from 'react-dom/client';\nimport App from './App';\nReactDOM.createRoot(document.getElementById('root')!).render(<App />);\n".into() },
                TemplateFile { path: "package.json".into(), content: "{\n  \"name\": \"{{project_name}}\",\n  \"version\": \"0.1.0\",\n  \"scripts\": { \"dev\": \"vite\", \"build\": \"vite build\" }\n}\n".into() },
                TemplateFile { path: "README.md".into(), content: "# {{project_name}}\n\n{{description}}\n".into() },
            ],
        },
        ProjectTemplate {
            id: "python-cli".into(),
            name: "Python CLI Tool".into(),
            description: "Argparse-based Python command line tool".into(),
            language: "Python".into(),
            tags: vec!["cli".into(), "python".into(), "tool".into()],
            structure: vec![
                TemplateFile { path: "main.py".into(), content: "#!/usr/bin/env python3\n\"\"\"{{description}}\"\"\"\nimport argparse\n\ndef main():\n    parser = argparse.ArgumentParser(description='{{description}}')\n    args = parser.parse_args()\n\nif __name__ == '__main__':\n    main()\n".into() },
                TemplateFile { path: "requirements.txt".into(), content: "".into() },
                TemplateFile { path: "README.md".into(), content: "# {{project_name}}\n\n{{description}}\n\n## Usage\n\n```bash\npython main.py\n```\n".into() },
            ],
        },
        ProjectTemplate {
            id: "tauri-app".into(),
            name: "Tauri Desktop App".into(),
            description: "Cross-platform desktop app with Tauri + React".into(),
            language: "Rust/TypeScript".into(),
            tags: vec!["desktop".into(), "tauri".into(), "rust".into()],
            structure: vec![
                TemplateFile { path: "README.md".into(), content: "# {{project_name}}\n\n{{description}}\n\n## Development\n\n```bash\ncargo tauri dev\n```\n".into() },
            ],
        },
        ProjectTemplate {
            id: "api-fastapi".into(),
            name: "FastAPI REST API".into(),
            description: "Python FastAPI backend with auto-generated docs".into(),
            language: "Python".into(),
            tags: vec!["api".into(), "backend".into(), "python".into()],
            structure: vec![
                TemplateFile { path: "main.py".into(), content: "from fastapi import FastAPI\n\napp = FastAPI(title='{{project_name}}', description='{{description}}')\n\n@app.get('/')\nasync def root():\n    return {'message': 'Hello from {{project_name}}'}\n".into() },
                TemplateFile { path: "requirements.txt".into(), content: "fastapi\nuvicorn[standard]\n".into() },
                TemplateFile { path: "README.md".into(), content: "# {{project_name}}\n\n{{description}}\n\n## Run\n\n```bash\nuvicorn main:app --reload\n```\n".into() },
            ],
        },
        ProjectTemplate {
            id: "rust-lib".into(),
            name: "Rust Library".into(),
            description: "Rust crate with tests and documentation".into(),
            language: "Rust".into(),
            tags: vec!["rust".into(), "library".into(), "crate".into()],
            structure: vec![
                TemplateFile { path: "src/lib.rs".into(), content: "//! {{description}}\n\n/// Example function\npub fn hello() -> &'static str {\n    \"Hello from {{project_name}}!\"\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn it_works() {\n        assert_eq!(hello(), \"Hello from {{project_name}}!\");\n    }\n}\n".into() },
                TemplateFile { path: "README.md".into(), content: "# {{project_name}}\n\n{{description}}\n".into() },
            ],
        },
    ])
}

/// Scaffolds project files on disk, inits git, creates GitHub repo, pushes
#[tauri::command]
pub async fn build_and_push_project(
    req: BuildProjectRequest,
) -> Result<BuildResult, String> {
    let project_dir = format!("{}/{}", req.output_dir.trim_end_matches('/'), req.project_name);

    // 1. Create directory
    fs::create_dir_all(&project_dir)
        .map_err(|e| format!("Failed to create project dir: {e}"))?;

    // 2. Write all files
    for file in &req.generated_files {
        let full_path = format!("{}/{}", project_dir, file.path);
        // Create parent dirs
        if let Some(parent) = std::path::Path::new(&full_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir {}: {e}", parent.display()))?;
        }
        fs::write(&full_path, &file.content)
            .map_err(|e| format!("Failed to write {}: {e}", file.path))?;
    }

    // 3. Write .gitignore
    let gitignore = "node_modules/\n.env\n*.log\ntarget/\n__pycache__/\n*.pyc\n.DS_Store\n";
    fs::write(format!("{project_dir}/.gitignore"), gitignore)
        .map_err(|e| format!("Failed to write .gitignore: {e}"))?;

    // 4. Git init
    let git_init = Command::new("git")
        .args(["init"])
        .current_dir(&project_dir)
        .output()
        .map_err(|e| e.to_string())?;

    if !git_init.status.success() {
        return Err(String::from_utf8_lossy(&git_init.stderr).to_string());
    }

    // 5. Initial commit
    Command::new("git").args(["add", "-A"]).current_dir(&project_dir).output().map_err(|e| e.to_string())?;
    let commit = Command::new("git")
        .args(["commit", "-m", &format!("Initial commit: {}", req.description)])
        .current_dir(&project_dir)
        .output()
        .map_err(|e| e.to_string())?;

    if !commit.status.success() {
        return Err(String::from_utf8_lossy(&commit.stderr).to_string());
    }

    // 6. Create GitHub repo and push using gh CLI
    let visibility = if req.private_repo { "--private" } else { "--public" };
    let gh_create = Command::new("gh")
        .args([
            "repo", "create", &req.project_name,
            "--source", &project_dir,
            "--remote", "origin",
            "--push",
            visibility,
            "--description", &req.description,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !gh_create.status.success() {
        let err = String::from_utf8_lossy(&gh_create.stderr).to_string();
        // Still return success for local - just note the push failed
        return Ok(BuildResult {
            success: true,
            local_path: project_dir,
            repo_url: None,
            message: format!("Project created locally. GitHub push failed: {err}"),
        });
    }

    // 7. Get the repo URL
    let repo_url_output = Command::new("gh")
        .args(["repo", "view", &req.project_name, "--json", "url", "--jq", ".url"])
        .output()
        .map_err(|e| e.to_string())?;

    let repo_url = String::from_utf8_lossy(&repo_url_output.stdout).trim().to_string();

    Ok(BuildResult {
        success: true,
        local_path: project_dir,
        repo_url: Some(repo_url),
        message: format!("✅ {} created and pushed to GitHub!", req.project_name),
    })
}

/// Apply template placeholders to file content
pub fn apply_template(content: &str, project_name: &str, description: &str) -> String {
    content
        .replace("{{project_name}}", project_name)
        .replace("{{description}}", description)
}

/// Build GeneratedFiles from a template
#[tauri::command]
pub async fn template_to_files(
    template_id: String,
    project_name: String,
    description: String,
) -> Result<Vec<GeneratedFile>, String> {
    let templates = get_templates().await?;
    let template = templates.into_iter()
        .find(|t| t.id == template_id)
        .ok_or_else(|| format!("Template '{template_id}' not found"))?;

    let files = template.structure.into_iter().map(|f| GeneratedFile {
        path: f.path,
        content: apply_template(&f.content, &project_name, &description),
    }).collect();

    Ok(files)
}
