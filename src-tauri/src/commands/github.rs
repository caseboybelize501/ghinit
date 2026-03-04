use std::process::Command;
use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repo {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub private: bool,
    pub url: String,
    pub default_branch: String,
    pub stars: u32,
    pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PullRequest {
    pub number: u32,
    pub title: String,
    pub state: String,
    pub author: String,
    pub url: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Issue {
    pub number: u32,
    pub title: String,
    pub state: String,
    pub author: String,
    pub url: String,
    pub labels: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommitResult {
    pub success: bool,
    pub sha: Option<String>,
    pub message: String,
}

// ── Auth ──────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn gh_auth_status() -> Result<serde_json::Value, String> {
    // First check if gh exists at all
    let which = Command::new("gh").args(["--version"]).output();
    if which.is_err() {
        return Ok(serde_json::json!({"loggedIn": false, "error": "gh not found"}));
    }

    // gh auth status exits with 0 if logged in, 1 if not
    let output = Command::new("gh")
        .args(["auth", "status"])
        .output()
        .map_err(|e| e.to_string())?;

    let combined = format!(
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if output.status.success() || combined.contains("Logged in") {
        // More robust username extraction: look for "as " or "account " and take the following word before any (
        let user = combined.lines()
            .find(|l| l.contains("Logged in to"))
            .and_then(|l| {
                if let Some(pos) = l.find(" as ") {
                    Some(&l[pos + 4..])
                } else if let Some(pos) = l.find(" account ") {
                    Some(&l[pos + 9..])
                } else {
                    l.split_whitespace().last()
                }
            })
            .and_then(|s| s.split_whitespace().next())
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_').to_string());

        Ok(serde_json::json!({
            "loggedIn": true,
            "user": user
        }))
    } else {
        Ok(serde_json::json!({"loggedIn": false}))
    }
}

#[tauri::command]
pub async fn gh_auth_login() -> Result<String, String> {
    // Opens browser-based login flow
    let output = Command::new("gh")
        .args(["auth", "login", "--web", "--hostname", "github.com"])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ── Repo Operations ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn gh_list_repos(limit: Option<u32>) -> Result<Vec<Repo>, String> {
    let limit = limit.unwrap_or(30).to_string();
    let output = Command::new("gh")
        .args([
            "repo", "list",
            "--json", "name,nameWithOwner,description,isPrivate,url,defaultBranchRef,stargazerCount,primaryLanguage",
            "--limit", &limit,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Parse error: {e}"))?;

    let repos = json.as_array().unwrap_or(&vec![]).iter().map(|r| Repo {
        name: r["name"].as_str().unwrap_or("").to_string(),
        full_name: r["nameWithOwner"].as_str().unwrap_or("").to_string(),
        description: r["description"].as_str().map(|s| s.to_string()),
        private: r["isPrivate"].as_bool().unwrap_or(false),
        url: r["url"].as_str().unwrap_or("").to_string(),
        default_branch: r["defaultBranchRef"]["name"].as_str().unwrap_or("main").to_string(),
        stars: r["stargazerCount"].as_u64().unwrap_or(0) as u32,
        language: r["primaryLanguage"]["name"].as_str().map(|s| s.to_string()),
    }).collect();

    Ok(repos)
}

#[tauri::command]
pub async fn gh_create_repo(
    name: String,
    description: Option<String>,
    private: bool,
    local_path: String,
) -> Result<Repo, String> {
    let mut args = vec![
        "repo".to_string(), "create".to_string(), name.clone(),
        "--source".to_string(), local_path.clone(),
        "--remote".to_string(), "origin".to_string(),
        "--push".to_string(),
    ];

    if private {
        args.push("--private".into());
    } else {
        args.push("--public".into());
    }

    if let Some(desc) = &description {
        args.push("--description".into());
        args.push(desc.clone());
    }

    let output = Command::new("gh")
        .args(&args)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    // Fetch the created repo info
    let repo_output = Command::new("gh")
        .args(["repo", "view", &name, "--json",
               "name,nameWithOwner,description,isPrivate,url,defaultBranchRef,stargazerCount,primaryLanguage"])
        .output()
        .map_err(|e| e.to_string())?;

    let r: serde_json::Value = serde_json::from_slice(&repo_output.stdout)
        .map_err(|e| format!("Parse error: {e}"))?;

    Ok(Repo {
        name: r["name"].as_str().unwrap_or(&name).to_string(),
        full_name: r["nameWithOwner"].as_str().unwrap_or("").to_string(),
        description: r["description"].as_str().map(|s| s.to_string()),
        private,
        url: r["url"].as_str().unwrap_or("").to_string(),
        default_branch: r["defaultBranchRef"]["name"].as_str().unwrap_or("main").to_string(),
        stars: 0,
        language: r["primaryLanguage"]["name"].as_str().map(|s| s.to_string()),
    })
}

#[tauri::command]
pub async fn gh_clone_repo(full_name: String, target_dir: String) -> Result<String, String> {
    let output = Command::new("gh")
        .args(["repo", "clone", &full_name, &target_dir])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(format!("Cloned {full_name} to {target_dir}"))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub async fn gh_browse_repo(full_name: String) -> Result<Vec<serde_json::Value>, String> {
    // List root-level files via gh api
    let output = Command::new("gh")
        .args(["api", &format!("repos/{full_name}/contents/")])
        .output()
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Parse error: {e}"))?;

    Ok(json.as_array().cloned().unwrap_or_default())
}

// ── Git / Commit ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn git_commit_and_push(
    repo_path: String,
    message: String,
    files: Option<Vec<String>>, // None = git add -A
) -> Result<CommitResult, String> {
    // Stage files
    let add_args: Vec<String> = match &files {
        Some(f) if !f.is_empty() => {
            let mut a = vec!["add".to_string()];
            a.extend(f.clone());
            a
        }
        _ => vec!["add".to_string(), "-A".to_string()],
    };

    let add = Command::new("git")
        .args(&add_args)
        .current_dir(&repo_path)
        .output()
        .map_err(|e| e.to_string())?;

    if !add.status.success() {
        return Err(String::from_utf8_lossy(&add.stderr).to_string());
    }

    // Commit
    let commit = Command::new("git")
        .args(["commit", "-m", &message])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| e.to_string())?;

    if !commit.status.success() {
        let stderr = String::from_utf8_lossy(&commit.stderr).to_string();
        if stderr.contains("nothing to commit") {
            return Ok(CommitResult { success: true, sha: None, message: "Nothing to commit".into() });
        }
        return Err(stderr);
    }

    // Push
    let push = Command::new("git")
        .args(["push"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| e.to_string())?;

    if !push.status.success() {
        return Err(String::from_utf8_lossy(&push.stderr).to_string());
    }

    // Get latest SHA
    let sha_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| e.to_string())?;

    let sha = String::from_utf8_lossy(&sha_output.stdout).trim().to_string();

    Ok(CommitResult {
        success: true,
        sha: Some(sha),
        message: format!("Committed and pushed: {message}"),
    })
}

// ── Pull Requests ─────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn gh_list_prs(full_name: String, state: Option<String>) -> Result<Vec<PullRequest>, String> {
    let state = state.unwrap_or("open".into());
    let output = Command::new("gh")
        .args([
            "pr", "list",
            "--repo", &full_name,
            "--state", &state,
            "--json", "number,title,state,author,url,createdAt",
            "--limit", "50",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Parse error: {e}"))?;

    let prs = json.as_array().unwrap_or(&vec![]).iter().map(|p| PullRequest {
        number: p["number"].as_u64().unwrap_or(0) as u32,
        title: p["title"].as_str().unwrap_or("").to_string(),
        state: p["state"].as_str().unwrap_or("").to_string(),
        author: p["author"]["login"].as_str().unwrap_or("").to_string(),
        url: p["url"].as_str().unwrap_or("").to_string(),
        created_at: p["createdAt"].as_str().unwrap_or("").to_string(),
    }).collect();

    Ok(prs)
}

#[tauri::command]
pub async fn gh_create_pr(
    full_name: String,
    title: String,
    body: Option<String>,
    base: Option<String>,
    draft: bool,
) -> Result<PullRequest, String> {
    let base = base.unwrap_or("main".into());
    let mut args = vec![
        "pr".to_string(), "create".to_string(),
        "--repo".to_string(), full_name.clone(),
        "--title".to_string(), title.clone(),
        "--base".to_string(), base,
        "--json".to_string(), "number,title,state,author,url,createdAt".to_string(),
    ];

    if let Some(b) = &body {
        args.push("--body".into());
        args.push(b.clone());
    } else {
        args.push("--body".into());
        args.push("".into());
    }

    if draft { args.push("--draft".into()); }

    let output = Command::new("gh")
        .args(&args)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let p: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Parse error: {e}"))?;

    Ok(PullRequest {
        number: p["number"].as_u64().unwrap_or(0) as u32,
        title: p["title"].as_str().unwrap_or(&title).to_string(),
        state: p["state"].as_str().unwrap_or("open").to_string(),
        author: p["author"]["login"].as_str().unwrap_or("").to_string(),
        url: p["url"].as_str().unwrap_or("").to_string(),
        created_at: p["createdAt"].as_str().unwrap_or("").to_string(),
    })
}

// ── Issues ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn gh_list_issues(full_name: String, state: Option<String>) -> Result<Vec<Issue>, String> {
    let state = state.unwrap_or("open".into());
    let output = Command::new("gh")
        .args([
            "issue", "list",
            "--repo", &full_name,
            "--state", &state,
            "--json", "number,title,state,author,url,labels",
            "--limit", "50",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Parse error: {e}"))?;

    let issues = json.as_array().unwrap_or(&vec![]).iter().map(|i| Issue {
        number: i["number"].as_u64().unwrap_or(0) as u32,
        title: i["title"].as_str().unwrap_or("").to_string(),
        state: i["state"].as_str().unwrap_or("").to_string(),
        author: i["author"]["login"].as_str().unwrap_or("").to_string(),
        url: i["url"].as_str().unwrap_or("").to_string(),
        labels: i["labels"].as_array().unwrap_or(&vec![])
            .iter().filter_map(|l| l["name"].as_str().map(|s| s.to_string())).collect(),
    }).collect();

    Ok(issues)
}

#[tauri::command]
pub async fn gh_create_issue(
    full_name: String,
    title: String,
    body: Option<String>,
    labels: Option<Vec<String>>,
) -> Result<Issue, String> {
    let mut args = vec![
        "issue".to_string(), "create".to_string(),
        "--repo".to_string(), full_name.clone(),
        "--title".to_string(), title.clone(),
        "--json".to_string(), "number,title,state,author,url,labels".to_string(),
    ];

    if let Some(b) = &body {
        args.push("--body".into());
        args.push(b.clone());
    }

    if let Some(lbls) = &labels {
        for l in lbls {
            args.push("--label".into());
            args.push(l.clone());
        }
    }

    let output = Command::new("gh").args(&args).output().map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let i: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Parse error: {e}"))?;

    Ok(Issue {
        number: i["number"].as_u64().unwrap_or(0) as u32,
        title: i["title"].as_str().unwrap_or(&title).to_string(),
        state: "open".into(),
        author: i["author"]["login"].as_str().unwrap_or("").to_string(),
        url: i["url"].as_str().unwrap_or("").to_string(),
        labels: labels.unwrap_or_default(),
    })
}
