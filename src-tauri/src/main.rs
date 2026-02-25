// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands {
    pub mod qwen;
    pub mod github;
    pub mod builder;
}

use commands::{qwen::*, github::*, builder::*};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Qwen / AI
            locate_qwen,
            qwen_generate,
            // GitHub
            gh_auth_status,
            gh_auth_login,
            gh_list_repos,
            gh_create_repo,
            gh_clone_repo,
            gh_browse_repo,
            git_commit_and_push,
            gh_list_prs,
            gh_create_pr,
            gh_list_issues,
            gh_create_issue,
            // Builder
            get_templates,
            build_and_push_project,
            template_to_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
