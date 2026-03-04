use serde::{Serialize, Deserialize};
use tauri::command;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectRecord {
    pub id: String,
    pub name: String,
    pub path: String,
    pub remote_url: Option<String>,
    pub tech_stack: Vec<String>,
    pub last_modified: u64,
}

fn get_db_path() -> String {
    // Move DB out of watched src-tauri folder to prevent dev server crash loop
    "../.purposeforge/projects_db.json".to_string()
}

#[command]
pub async fn track_project(project: ProjectRecord) -> Result<(), String> {
    track_project_internal(project).await
}

pub async fn track_project_internal(project: ProjectRecord) -> Result<(), String> {
    let mut projects = list_tracked_projects().await?;
    
    // Update existing or add new
    if let Some(existing) = projects.iter_mut().find(|p| p.path == project.path) {
        *existing = project;
    } else {
        projects.push(project);
    }

    let json = serde_json::to_string_pretty(&projects)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    
    fs::write(get_db_path(), json)
        .map_err(|e| format!("Failed to write DB: {}", e))?;

    Ok(())
}

#[command]
pub async fn list_tracked_projects() -> Result<Vec<ProjectRecord>, String> {
    let path = get_db_path();
    let mut projects = if Path::new(&path).exists() {
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read DB: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse DB: {}", e))?
    } else {
        vec![]
    };

    // If empty, try basic discovery in the default CASE directory
    if projects.is_empty() {
        if let Ok(entries) = fs::read_dir("d:/Users/CASE/projects") {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        let name = entry.file_name().to_string_lossy().into_owned();
                        let path = entry.path().to_string_lossy().into_owned();
                        // Filter out common non-project dirs
                        if name == "purposeforge" || name == ".git" { continue; }
                        
                        let mut tech_stack = vec![];
                        if Path::new(&path).join("package.json").exists() { tech_stack.push("node".into()); }
                        if Path::new(&path).join("requirements.txt").exists() { tech_stack.push("python".into()); }
                        if Path::new(&path).join("Cargo.toml").exists() { tech_stack.push("rust".into()); }
                        if Path::new(&path).join("vite.config.ts").exists() { tech_stack.push("react".into()); }

                        projects.push(ProjectRecord {
                            id: uuid::Uuid::new_v4().to_string(),
                            name,
                            path,
                            remote_url: None,
                            tech_stack,
                            last_modified: metadata.modified().unwrap_or(SystemTime::now())
                                .duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        });
                    }
                }
            }
        }
    }

    Ok(projects)
}

#[command]
pub async fn update_project_status(id: String, _status: String) -> Result<(), String> {
    let mut projects = list_tracked_projects().await?;
    if let Some(project) = projects.iter_mut().find(|p| p.id == id) {
        project.last_modified = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let json = serde_json::to_string_pretty(&projects)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        fs::write(get_db_path(), json)
            .map_err(|e| format!("Failed to write DB: {}", e))?;
    }
    Ok(())
}
