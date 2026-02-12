//! Dev Server controller for detecting, starting, and stopping project dev servers

use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::services::process_manager::PROCESS_MANAGER;

#[derive(Debug, Deserialize)]
pub struct DetectRequest {
    pub repo_path: String,
}

#[derive(Debug, Serialize)]
pub struct DetectResponse {
    pub found: bool,
    pub package_json_dir: Option<String>,
    pub package_manager: Option<String>,
    pub has_dev_script: bool,
}

#[derive(Debug, Deserialize)]
pub struct StartRequest {
    pub repo_path: String,
    pub package_json_dir: String,
    pub package_manager: String,
}

#[derive(Debug, Serialize)]
pub struct StartResponse {
    pub success: bool,
    pub pid: Option<u32>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StopRequest {
    pub repo_path: String,
}

#[derive(Debug, Serialize)]
pub struct StopResponse {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    pub repo_path: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub running: bool,
}

/// Detect the package manager for a given directory
fn detect_package_manager(dir: &Path) -> String {
    if dir.join("bun.lockb").exists() || dir.join("bun.lock").exists() {
        "bun".to_string()
    } else if dir.join("pnpm-lock.yaml").exists() {
        "pnpm".to_string()
    } else if dir.join("yarn.lock").exists() {
        "yarn".to_string()
    } else {
        "npm".to_string()
    }
}

/// Check if package.json has a "dev" script
fn has_dev_script(dir: &Path) -> bool {
    let pkg_path = dir.join("package.json");
    if let Ok(content) = std::fs::read_to_string(&pkg_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            return json
                .get("scripts")
                .and_then(|s| s.get("dev"))
                .is_some();
        }
    }
    false
}

/// Detect package.json and package manager in a repo
#[debug_handler]
async fn detect(Json(params): Json<DetectRequest>) -> Result<Response> {
    let repo_path = Path::new(&params.repo_path);

    // Check root first
    if repo_path.join("package.json").exists() {
        let pm = detect_package_manager(repo_path);
        let has_dev = has_dev_script(repo_path);
        return format::json(DetectResponse {
            found: true,
            package_json_dir: Some(params.repo_path.clone()),
            package_manager: Some(pm),
            has_dev_script: has_dev,
        });
    }

    // Scan 1-level subdirectories
    if let Ok(entries) = std::fs::read_dir(repo_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str == "node_modules" || name_str == ".git" || name_str.starts_with('.') {
                continue;
            }
            if path.join("package.json").exists() {
                let pm = detect_package_manager(&path);
                let has_dev = has_dev_script(&path);
                return format::json(DetectResponse {
                    found: true,
                    package_json_dir: Some(path.to_string_lossy().to_string()),
                    package_manager: Some(pm),
                    has_dev_script: has_dev,
                });
            }
        }
    }

    format::json(DetectResponse {
        found: false,
        package_json_dir: None,
        package_manager: None,
        has_dev_script: false,
    })
}

/// Start the dev server
#[debug_handler]
async fn start(Json(params): Json<StartRequest>) -> Result<Response> {
    let args = vec!["run", "dev"];

    match PROCESS_MANAGER
        .spawn_devserver(
            &params.repo_path,
            &params.package_manager,
            &args,
            &params.package_json_dir,
        )
        .await
    {
        Ok(pid) => format::json(StartResponse {
            success: true,
            pid: Some(pid),
            error: None,
        }),
        Err(e) => format::json(StartResponse {
            success: false,
            pid: None,
            error: Some(e),
        }),
    }
}

/// Stop the dev server
#[debug_handler]
async fn stop(Json(params): Json<StopRequest>) -> Result<Response> {
    match PROCESS_MANAGER.kill_devserver(&params.repo_path).await {
        Ok(()) => format::json(StopResponse {
            success: true,
            error: None,
        }),
        Err(e) => format::json(StopResponse {
            success: false,
            error: Some(e),
        }),
    }
}

/// Get dev server status
#[debug_handler]
async fn status(Query(params): Query<StatusQuery>) -> Result<Response> {
    let running = PROCESS_MANAGER.is_devserver_running(&params.repo_path);
    format::json(StatusResponse { running })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/devserver")
        .add("/detect", post(detect))
        .add("/start", post(start))
        .add("/stop", post(stop))
        .add("/status", get(status))
}
