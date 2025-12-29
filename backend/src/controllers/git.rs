//! Git repository validation and branch listing controller

use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct ValidatePathRequest {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct ValidatePathResponse {
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BranchesQuery {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct BranchesResponse {
    pub branches: Vec<String>,
    pub current: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Validate if a path is a valid git repository
#[debug_handler]
async fn validate_path(Json(params): Json<ValidatePathRequest>) -> Result<Response> {
    let path = Path::new(&params.path);

    // Check if path exists
    if !path.exists() {
        return format::json(ValidatePathResponse {
            valid: false,
            error: Some("Path does not exist".to_string()),
        });
    }

    // Check if it's a directory
    if !path.is_dir() {
        return format::json(ValidatePathResponse {
            valid: false,
            error: Some("Path is not a directory".to_string()),
        });
    }

    // Check if it's a git repository by running git rev-parse
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(path)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                format::json(ValidatePathResponse {
                    valid: true,
                    error: None,
                })
            } else {
                format::json(ValidatePathResponse {
                    valid: false,
                    error: Some("Not a git repository".to_string()),
                })
            }
        }
        Err(e) => format::json(ValidatePathResponse {
            valid: false,
            error: Some(format!("Failed to check git repository: {}", e)),
        }),
    }
}

/// Get all branches in a git repository
#[debug_handler]
async fn get_branches(Query(params): Query<BranchesQuery>) -> Result<Response> {
    let path = Path::new(&params.path);

    // Check if path exists and is a git repo
    if !path.exists() || !path.is_dir() {
        return format::json(ErrorResponse {
            error: "Invalid path".to_string(),
        });
    }

    // Get current branch
    let current_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path)
        .output();

    let current_branch = current_output.ok().and_then(|o| {
        if o.status.success() {
            String::from_utf8(o.stdout)
                .ok()
                .map(|s| s.trim().to_string())
        } else {
            None
        }
    });

    // Get all local branches
    let branches_output = Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .current_dir(path)
        .output();

    match branches_output {
        Ok(output) => {
            if output.status.success() {
                let branches: Vec<String> = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                format::json(BranchesResponse {
                    branches,
                    current: current_branch,
                })
            } else {
                format::json(ErrorResponse {
                    error: String::from_utf8_lossy(&output.stderr).to_string(),
                })
            }
        }
        Err(e) => format::json(ErrorResponse {
            error: format!("Failed to list branches: {}", e),
        }),
    }
}

/// Fetch latest from remote
#[debug_handler]
async fn fetch(Json(params): Json<ValidatePathRequest>) -> Result<Response> {
    let path = Path::new(&params.path);

    if !path.exists() || !path.is_dir() {
        return format::json(ErrorResponse {
            error: "Invalid path".to_string(),
        });
    }

    let output = Command::new("git")
        .args(["fetch", "--all", "--prune"])
        .current_dir(path)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                format::json(serde_json::json!({ "success": true }))
            } else {
                format::json(ErrorResponse {
                    error: String::from_utf8_lossy(&output.stderr).to_string(),
                })
            }
        }
        Err(e) => format::json(ErrorResponse {
            error: format!("Failed to fetch: {}", e),
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct DetectPathRequest {
    pub marker_filename: String,
}

#[derive(Debug, Serialize)]
pub struct DetectPathResponse {
    pub found: bool,
    pub path: Option<String>,
}

/// Detect the full path of a directory by searching for a marker file
/// This is used as a workaround for browser security restrictions
#[debug_handler]
async fn detect_path(Json(params): Json<DetectPathRequest>) -> Result<Response> {
    let marker = &params.marker_filename;

    // Security check: marker must start with our prefix
    if !marker.starts_with(".clickup-orchestrator-path-") {
        return format::json(DetectPathResponse {
            found: false,
            path: None,
        });
    }

    // Search in common development directories
    let search_roots: Vec<PathBuf> = vec![
        dirs::home_dir(),
        dirs::document_dir(),
        dirs::desktop_dir(),
        Some(PathBuf::from("/tmp")),
        Some(PathBuf::from("/var/tmp")),
    ]
    .into_iter()
    .flatten()
    .collect();

    // Also add common dev folders
    let mut all_roots = search_roots.clone();
    if let Some(home) = dirs::home_dir() {
        let common_dev_folders = [
            "Projects",
            "projects",
            "Development",
            "development",
            "dev",
            "Dev",
            "code",
            "Code",
            "repos",
            "Repos",
            "src",
            "Documents/dev",
            "Documents/Development",
            "Documents/projects",
        ];
        for folder in common_dev_folders {
            all_roots.push(home.join(folder));
        }
    }

    // Search for the marker file
    for root in all_roots {
        if !root.exists() {
            continue;
        }

        // Walk the directory tree (max depth 10 to avoid going too deep)
        for entry in WalkDir::new(&root)
            .max_depth(10)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name().to_string_lossy() == *marker {
                // Found the marker file - return its parent directory
                if let Some(parent) = entry.path().parent() {
                    return format::json(DetectPathResponse {
                        found: true,
                        path: Some(parent.to_string_lossy().to_string()),
                    });
                }
            }
        }
    }

    format::json(DetectPathResponse {
        found: false,
        path: None,
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/git")
        .add("/validate-path", post(validate_path))
        .add("/branches", get(get_branches))
        .add("/fetch", post(fetch))
        .add("/detect-path", post(detect_path))
}
