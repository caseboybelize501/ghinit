use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use tauri::{AppHandle, Emitter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QwenLocation {
    pub found: bool,
    pub method: String, // "ollama" | "lmstudio" | "binary" | "none"
    pub path: Option<String>,
    pub model: Option<String>,
}

/// Searches for Qwen Coder in all known locations across Windows/macOS/Linux
#[tauri::command]
pub async fn locate_qwen() -> Result<QwenLocation, String> {
    // 1. Check Ollama first (most common)
    if let Ok(output) = Command::new("ollama").args(["list"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Look for any qwen model
        for line in stdout.lines() {
            let lower = line.to_lowercase();
            if lower.contains("qwen") && lower.contains("code") {
                let model = line.split_whitespace().next().unwrap_or("qwen2.5-coder").to_string();
                return Ok(QwenLocation {
                    found: true,
                    method: "ollama".into(),
                    path: which_binary("ollama"),
                    model: Some(model),
                });
            }
        }
        // Ollama exists but no qwen model — still report ollama is available
        if output.status.success() {
            return Ok(QwenLocation {
                found: false,
                method: "ollama_no_model".into(),
                path: which_binary("ollama"),
                model: None,
            });
        }
    }

    // 2. Check LM Studio local server (runs on port 1234)
    if let Ok(resp) = reqwest::get("http://localhost:1234/v1/models").await {
        if resp.status().is_success() {
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            if let Some(models) = json["data"].as_array() {
                for model in models {
                    let id = model["id"].as_str().unwrap_or("").to_lowercase();
                    if id.contains("qwen") {
                        return Ok(QwenLocation {
                            found: true,
                            method: "lmstudio".into(),
                            path: Some("http://localhost:1234".into()),
                            model: Some(model["id"].as_str().unwrap_or("qwen").to_string()),
                        });
                    }
                }
            }
        }
    }

    // 3. Search for raw binary / .gguf in common paths
    let search_paths = get_search_paths();
    for path in &search_paths {
        let p = std::path::Path::new(path);
        if p.exists() {
            return Ok(QwenLocation {
                found: true,
                method: "binary".into(),
                path: Some(path.clone()),
                model: None,
            });
        }
    }

    Ok(QwenLocation {
        found: false,
        method: "none".into(),
        path: None,
        model: None,
    })
}

fn which_binary(name: &str) -> Option<String> {
    Command::new("where")
        .arg(name)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().lines().next().unwrap_or("").to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            Command::new("which")
                .arg(name)
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
}

fn get_search_paths() -> Vec<String> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();

    vec![
        // Windows common paths
        format!("{home}\\AppData\\Local\\Programs\\Ollama\\ollama.exe"),
        format!("{home}\\.ollama\\models"),
        format!("{home}\\AppData\\Roaming\\LM Studio\\models"),
        // Generic binary names
        format!("{home}\\bin\\qwen-coder"),
        // Linux/macOS
        format!("{home}/.local/bin/qwen"),
        "/usr/local/bin/qwen".into(),
        "/opt/qwen/qwen".into(),
    ]
}

/// Stream a prompt to Qwen and emit tokens back to the frontend via Tauri events
#[tauri::command]
pub async fn qwen_generate(
    app: AppHandle,
    location: QwenLocation,
    prompt: String,
    system: Option<String>,
) -> Result<String, String> {
    match location.method.as_str() {
        "ollama" => {
            let model = location.model.unwrap_or("qwen2.5-coder:latest".into());
            stream_ollama(&app, &model, &prompt, system.as_deref()).await
        }
        "lmstudio" => {
            stream_openai_compat(
                &app,
                "http://localhost:1234/v1/chat/completions",
                &location.model.unwrap_or("qwen".into()),
                &prompt,
                system.as_deref(),
            ).await
        }
        _ => Err("Qwen not found. Please install via Ollama: `ollama pull qwen2.5-coder`".into()),
    }
}

async fn stream_ollama(
    app: &AppHandle,
    model: &str,
    prompt: &str,
    system: Option<&str>,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let mut body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": true
    });
    if let Some(sys) = system {
        body["system"] = serde_json::Value::String(sys.to_string());
    }

    let resp = client
        .post("http://localhost:11434/api/generate")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Ollama connection failed: {e}"))?;

    let mut full_response = String::new();
    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(token) = val["response"].as_str() {
                    full_response.push_str(token);
                    let _ = app.emit("qwen-token", token.to_string());
                }
                if val["done"].as_bool().unwrap_or(false) {
                    let _ = app.emit("qwen-done", full_response.clone());
                }
            }
        }
    }
    Ok(full_response)
}

async fn stream_openai_compat(
    app: &AppHandle,
    endpoint: &str,
    model: &str,
    prompt: &str,
    system: Option<&str>,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let mut messages = vec![];
    if let Some(sys) = system {
        messages.push(serde_json::json!({"role": "system", "content": sys}));
    }
    messages.push(serde_json::json!({"role": "user", "content": prompt}));

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": true
    });

    let resp = client
        .post(endpoint)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("LM Studio connection failed: {e}"))?;

    let mut full_response = String::new();
    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            let line = line.trim_start_matches("data: ");
            if line == "[DONE]" { break; }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(token) = val["choices"][0]["delta"]["content"].as_str() {
                    full_response.push_str(token);
                    let _ = app.emit("qwen-token", token.to_string());
                }
            }
        }
    }
    let _ = app.emit("qwen-done", full_response.clone());
    Ok(full_response)
}
