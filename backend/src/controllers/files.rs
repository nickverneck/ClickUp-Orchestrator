//! File system controller for browsing and editing files

use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileNode>>,
}

#[derive(Debug, Deserialize)]
pub struct TreeQuery {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Get file tree for a directory
#[debug_handler]
async fn get_tree(Query(params): Query<TreeQuery>) -> Result<Response> {
    let root_path = Path::new(&params.path);

    if !root_path.exists() {
        return format::json(ErrorResponse {
            error: "Path does not exist".to_string(),
        });
    }

    if !root_path.is_dir() {
        return format::json(ErrorResponse {
            error: "Path is not a directory".to_string(),
        });
    }

    match build_tree(root_path, 0, 3) {
        Ok(nodes) => format::json(nodes),
        Err(e) => format::json(ErrorResponse {
            error: format!("Failed to read directory: {}", e),
        }),
    }
}

fn build_tree(path: &Path, depth: usize, max_depth: usize) -> std::io::Result<Vec<FileNode>> {
    let mut nodes = Vec::new();

    // Skip hidden files and common non-essential directories
    let skip_dirs = [
        "node_modules",
        ".git",
        "target",
        ".svelte-kit",
        "dist",
        "build",
        ".next",
        "__pycache__",
        ".venv",
        "venv",
    ];

    let mut entries: Vec<_> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            // Skip hidden files (starting with .) except for important config files
            if name.starts_with('.') {
                return false;
            }
            // Skip node_modules and other large directories
            if e.path().is_dir() && skip_dirs.contains(&name.as_str()) {
                return false;
            }
            true
        })
        .collect();

    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| {
        let a_is_dir = a.path().is_dir();
        let b_is_dir = b.path().is_dir();
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a
                .file_name()
                .to_string_lossy()
                .to_lowercase()
                .cmp(&b.file_name().to_string_lossy().to_lowercase()),
        }
    });

    for entry in entries {
        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry_path.is_dir();

        let children = if is_dir && depth < max_depth {
            match build_tree(&entry_path, depth + 1, max_depth) {
                Ok(c) => Some(c),
                Err(_) => Some(Vec::new()),
            }
        } else if is_dir {
            // Placeholder for unexpanded directories
            Some(Vec::new())
        } else {
            None
        };

        nodes.push(FileNode {
            name,
            path: entry_path.to_string_lossy().to_string(),
            is_directory: is_dir,
            children,
        });
    }

    Ok(nodes)
}

#[derive(Debug, Deserialize)]
pub struct ContentQuery {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct FileContent {
    pub content: String,
    pub language: String,
    pub encoding: String,
}

/// Get file content
#[debug_handler]
async fn get_content(Query(params): Query<ContentQuery>) -> Result<Response> {
    let file_path = Path::new(&params.path);

    if !file_path.exists() {
        return format::json(ErrorResponse {
            error: "File does not exist".to_string(),
        });
    }

    if !file_path.is_file() {
        return format::json(ErrorResponse {
            error: "Path is not a file".to_string(),
        });
    }

    // Check file size (limit to 5MB)
    if let Ok(metadata) = fs::metadata(file_path) {
        if metadata.len() > 5 * 1024 * 1024 {
            return format::json(ErrorResponse {
                error: "File is too large (max 5MB)".to_string(),
            });
        }
    }

    match fs::read_to_string(file_path) {
        Ok(content) => {
            let language = get_language_from_path(file_path);
            format::json(FileContent {
                content,
                language,
                encoding: "utf-8".to_string(),
            })
        }
        Err(e) => format::json(ErrorResponse {
            error: format!("Failed to read file: {}", e),
        }),
    }
}

fn get_language_from_path(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "ts" | "tsx" => "typescript".to_string(),
        "js" | "jsx" => "javascript".to_string(),
        "svelte" => "html".to_string(),
        "html" | "htm" => "html".to_string(),
        "css" => "css".to_string(),
        "scss" => "scss".to_string(),
        "json" => "json".to_string(),
        "md" | "markdown" => "markdown".to_string(),
        "rs" => "rust".to_string(),
        "py" => "python".to_string(),
        "go" => "go".to_string(),
        "yaml" | "yml" => "yaml".to_string(),
        "toml" => "toml".to_string(),
        "sql" => "sql".to_string(),
        "sh" | "bash" => "shell".to_string(),
        "xml" => "xml".to_string(),
        _ => "plaintext".to_string(),
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveContentRequest {
    pub path: String,
    pub content: String,
}

/// Save file content
#[debug_handler]
async fn save_content(Json(params): Json<SaveContentRequest>) -> Result<Response> {
    let file_path = Path::new(&params.path);

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            return format::json(ErrorResponse {
                error: "Parent directory does not exist".to_string(),
            });
        }
    }

    match fs::write(file_path, &params.content) {
        Ok(()) => format::json(serde_json::json!({ "success": true })),
        Err(e) => format::json(ErrorResponse {
            error: format!("Failed to save file: {}", e),
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateFileRequest {
    pub path: String,
    pub is_directory: bool,
}

/// Create a new file or directory
#[debug_handler]
async fn create_file(Json(params): Json<CreateFileRequest>) -> Result<Response> {
    let file_path = Path::new(&params.path);

    if file_path.exists() {
        return format::json(ErrorResponse {
            error: "Path already exists".to_string(),
        });
    }

    let result = if params.is_directory {
        fs::create_dir_all(file_path)
    } else {
        // Create parent directory if needed
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    return format::json(ErrorResponse {
                        error: format!("Failed to create parent directory: {}", e),
                    });
                }
            }
        }
        fs::write(file_path, "")
    };

    match result {
        Ok(()) => format::json(serde_json::json!({ "success": true })),
        Err(e) => format::json(ErrorResponse {
            error: format!("Failed to create: {}", e),
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    pub path: String,
}

/// Delete a file or directory
#[debug_handler]
async fn delete_file(Query(params): Query<DeleteQuery>) -> Result<Response> {
    let file_path = Path::new(&params.path);

    if !file_path.exists() {
        return format::json(ErrorResponse {
            error: "Path does not exist".to_string(),
        });
    }

    let result = if file_path.is_dir() {
        fs::remove_dir_all(file_path)
    } else {
        fs::remove_file(file_path)
    };

    match result {
        Ok(()) => format::json(serde_json::json!({ "success": true })),
        Err(e) => format::json(ErrorResponse {
            error: format!("Failed to delete: {}", e),
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct RenameRequest {
    pub old_path: String,
    pub new_path: String,
}

/// Rename/move a file or directory
#[debug_handler]
async fn rename_file(Json(params): Json<RenameRequest>) -> Result<Response> {
    let old_path = Path::new(&params.old_path);
    let new_path = Path::new(&params.new_path);

    if !old_path.exists() {
        return format::json(ErrorResponse {
            error: "Source path does not exist".to_string(),
        });
    }

    if new_path.exists() {
        return format::json(ErrorResponse {
            error: "Destination path already exists".to_string(),
        });
    }

    match fs::rename(old_path, new_path) {
        Ok(()) => format::json(serde_json::json!({ "success": true })),
        Err(e) => format::json(ErrorResponse {
            error: format!("Failed to rename: {}", e),
        }),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/files")
        .add("/tree", get(get_tree))
        .add("/content", get(get_content))
        .add("/content", put(save_content))
        .add("/create", post(create_file))
        .add("/", delete(delete_file))
        .add("/rename", post(rename_file))
}
