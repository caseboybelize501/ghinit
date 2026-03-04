use std::fs;

#[tauri::command]
pub async fn modify_file(path: String, target_content: String, replacement: String) -> Result<(), String> {
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let updated_content = content.replace(&target_content, &replacement);

    fs::write(&path, updated_content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn apply_diff(path: String, diff: String) -> Result<(), String> {
    // TODO: Implement patch application or AI-driven merging
    println!("Applying diff to {}: {}", path, diff);
    Ok(())
}

#[tauri::command]
pub async fn read_file_content(path: String) -> Result<String, String> {
    fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))
}
