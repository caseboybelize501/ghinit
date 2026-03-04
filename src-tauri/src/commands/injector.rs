use tauri::command;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tech_stack: Vec<String>,
    pub files: Vec<ModuleFileMapping>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleFileMapping {
    pub src: String,
    pub dest: String,
}

#[command]
pub async fn list_available_modules() -> Result<Vec<ModuleDefinition>, String> {
    let modules_dir = "d:/Users/CASE/projects/purposeforge/.purposeforge/modules";
    if !Path::new(modules_dir).exists() {
        return Ok(vec![]);
    }

    let mut result = vec![];
    if let Ok(entries) = fs::read_dir(modules_dir) {
        for entry in entries.flatten() {
            let manifest_path = entry.path().join("manifest.json");
            if manifest_path.exists() {
                if let Ok(content) = fs::read_to_string(manifest_path) {
                    if let Ok(def) = serde_json::from_str::<ModuleDefinition>(&content) {
                        result.push(def);
                    }
                }
            }
        }
    }
    Ok(result)
}

#[command]
pub async fn inject_module(project_path: String, module_id: String) -> Result<(), String> {
    let modules_dir = "d:/Users/CASE/projects/purposeforge/.purposeforge/modules";
    let manifest_path = format!("{}/{}/manifest.json", modules_dir, module_id);
    
    if !Path::new(&manifest_path).exists() {
        return Err(format!("Module {} not found", module_id));
    }

    let content = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let def: ModuleDefinition = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    for mapping in def.files {
        let src_path = format!("{}/{}/src/{}", modules_dir, module_id, mapping.src);
        let dest_path = format!("{}/{}", project_path, mapping.dest);

        // Create parent dirs
        if let Some(parent) = Path::new(&dest_path).parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        fs::copy(src_path, dest_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}
