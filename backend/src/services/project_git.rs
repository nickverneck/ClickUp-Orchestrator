//! Git operations for projects
//! Handles cloning, initializing, and validating git repositories

use std::path::Path;
use std::process::Command;

/// Initialize a new git repository at the specified path
pub async fn init_repo(path: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let output = Command::new("git")
        .args(&["init"])
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to execute git init: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "git init failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Clone a repository from GitHub to the specified path
pub async fn clone_repo(url: &str, path: &str) -> Result<(), String> {
    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directories: {}", e))?;
        }
    }

    let output = Command::new("git")
        .args(&["clone", url, path])
        .output()
        .map_err(|e| format!("Failed to execute git clone: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "git clone failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Validate that the specified path is a valid git repository
pub async fn validate_repo(path: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        return Err(format!("Repository path does not exist: {}", path));
    }

    let git_dir = Path::new(path).join(".git");
    if !git_dir.exists() {
        return Err(format!("Not a valid git repository (no .git directory): {}", path));
    }

    // Try to run a git command to verify it's a valid repo
    let output = Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to execute git command: {}", e))?;

    if !output.status.success() {
        return Err("Not a valid git repository".to_string());
    }

    Ok(())
}

/// Get the current branch of a repository
pub async fn get_current_branch(path: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to execute git command: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to get current branch: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Create a new worktree for a task
pub async fn create_worktree(
    repo_path: &str,
    worktree_path: &str,
    branch: &str,
) -> Result<(), String> {
    if !Path::new(repo_path).exists() {
        return Err(format!("Repository path does not exist: {}", repo_path));
    }

    // Create parent directory for worktree
    if let Some(parent) = Path::new(worktree_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create worktree parent directory: {}", e))?;
    }

    let output = Command::new("git")
        .args(&["worktree", "add", worktree_path, branch])
        .current_dir(repo_path)
        .output()
        .map_err(|e| format!("Failed to execute git worktree command: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to create worktree: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Remove a worktree
pub async fn remove_worktree(repo_path: &str, worktree_path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(&["worktree", "remove", worktree_path])
        .current_dir(repo_path)
        .output()
        .map_err(|e| format!("Failed to execute git worktree remove: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to remove worktree: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}
