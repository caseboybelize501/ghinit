use tauri::{AppHandle, Emitter};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;

#[tauri::command]
pub async fn open_project_folder(path: String) -> Result<(), String> {
    // Windows explorer expects backslashes, otherwise it opens Documents
    let win_path = path.replace("/", "\\");
    Command::new("explorer")
        .arg(&win_path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn open_in_vscode(path: String) -> Result<(), String> {
    Command::new("cmd")
        .args(["/C", "code", &path])
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| format!("Failed to open VS Code. Is it in your PATH? Error: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn open_in_cursor(path: String) -> Result<(), String> {
    let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Local".to_string());
    let cursor_exe = format!("{}\\Programs\\cursor\\Cursor.exe", localappdata);
    
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| "D:\\Users\\CASE\\AppData\\Roaming".to_string());
    let cursor_lnk = format!("{}\\Microsoft\\Windows\\Start Menu\\Programs\\Cursor\\Cursor.lnk", appdata);

    if std::path::Path::new(&cursor_exe).exists() {
        Command::new(&cursor_exe)
            .arg(&path)
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|e| format!("Failed to open Cursor from LocalAppData. Error: {}", e))?;
    } else if std::path::Path::new(&cursor_lnk).exists() {
        Command::new("cmd")
            .args(["/C", "start", "", &cursor_lnk, &path])
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|e| format!("Failed to open Cursor from Roaming shortcut. Error: {}", e))?;
    } else {
        // Fallback to cmd
        Command::new("cmd")
            .args(["/C", "cursor", &path])
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|e| format!("Failed to open Cursor. Is it in your PATH? Error: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn open_in_terminal(path: String) -> Result<(), String> {
    Command::new("cmd")
        .args(["/C", "start", "cmd", "/K", "cd /d", &path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn run_antigravity(path: String) -> Result<(), String> {
    Command::new("cmd")
        .args(["/C", "antigravity", &path])
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| format!("Failed to open Antigravity. Is it in your PATH? Error: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn copy_to_clipboard(text: String) -> Result<(), String> {
    // Windows clipboard hack
    let mut child = Command::new("clip")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    }
    child.wait().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn execute_task(cwd: String, command_str: String) -> Result<String, String> {
    let output = Command::new("cmd")
        .args(&["/C", &command_str])
        .current_dir(&cwd)
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| format!("Failed to execute: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub async fn stream_task(app: AppHandle, cwd: String, command_str: String, task_id: String) -> Result<(), String> {
    let app_meta = app.clone();
    let task_id_meta = task_id.clone();

    std::thread::spawn(move || {
        println!("[Executor] Spawning task: {} in {}", command_str, cwd);
        let spawn_result = Command::new("cmd")
            .args(&["/C", &command_str])
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(0x08000000)
            .spawn();

        let mut child = match spawn_result {
            Ok(c) => c,
            Err(e) => {
                let _ = app_meta.emit(&format!("task-err-{}", task_id_meta), format!("Failed to spawn: {}", e));
                let _ = app_meta.emit(&format!("task-done-{}", task_id_meta), false);
                return;
            }
        };

        let stdout = child.stdout.take().expect("Failed to open stdout");
        let stderr = child.stderr.take().expect("Failed to open stderr");
        
        let app_clone = app.clone();
        let task_id_clone = task_id.clone();
        
        // Handle stdout
        let stdout_thread = std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().flatten() {
                let _ = app_clone.emit(&format!("task-out-{}", task_id_clone), line);
            }
        });

        let app_clone2 = app.clone();
        let task_id_clone2 = task_id.clone();
        // Handle stderr
        let stderr_thread = std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                let _ = app_clone2.emit(&format!("task-err-{}", task_id_clone2), line);
            }
        });

        stdout_thread.join().unwrap();
        stderr_thread.join().unwrap();
        
        let status = child.wait().expect("Failed to wait on child");
        let _ = app.emit(&format!("task-done-{}", task_id), status.success());
    });
    
    Ok(())
}

#[tauri::command]
pub async fn get_environment_info() -> Result<String, String> {
    Ok("Windows/PowerShell ready".to_string())
}
