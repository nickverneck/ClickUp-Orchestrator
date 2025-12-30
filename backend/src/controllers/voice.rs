//! Voice Assistant controller for saving screenshots and spawning BA agent

use crate::models::_entities::settings;
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[derive(Debug, Deserialize)]
pub struct SaveScreenshotRequest {
    /// Base64 encoded image data (without data URL prefix)
    pub image_data: String,
    /// Optional filename, will generate one if not provided
    pub filename: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SaveScreenshotResponse {
    pub filepath: String,
    pub filename: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Claude,
    Codex,
    Gemini,
}

impl Default for AgentType {
    fn default() -> Self {
        AgentType::Claude
    }
}

#[derive(Debug, Deserialize)]
pub struct GenerateTasksRequest {
    /// The transcription text from voice recording
    pub transcript: String,
    /// List of screenshot filepaths (relative to repo)
    pub screenshots: Vec<String>,
    /// Which agent to use (claude, codex, gemini)
    #[serde(default)]
    pub agent: AgentType,
}

#[derive(Debug, Serialize)]
pub struct GenerateTasksResponse {
    pub success: bool,
    pub message: String,
    pub session_id: Option<String>,
}

/// Helper to get a setting value
async fn get_setting(db: &sea_orm::DatabaseConnection, key: &str) -> Option<String> {
    settings::Entity::find()
        .filter(settings::Column::Key.eq(key))
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|s| s.value)
        .filter(|v| !v.is_empty())
}

/// Save a screenshot to the temp_imgs folder in the target repo
#[debug_handler]
async fn save_screenshot(
    State(ctx): State<AppContext>,
    Json(params): Json<SaveScreenshotRequest>,
) -> Result<Response> {
    // Get target repo path from settings
    let repo_path = get_setting(&ctx.db, "target_repo_path")
        .await
        .ok_or_else(|| Error::BadRequest("Target repo path not configured".to_string()))?;

    // Create temp_imgs directory if it doesn't exist
    let temp_imgs_path = PathBuf::from(&repo_path).join("temp_imgs");
    tokio::fs::create_dir_all(&temp_imgs_path)
        .await
        .map_err(|e| Error::BadRequest(format!("Failed to create temp_imgs directory: {}", e)))?;

    // Generate filename if not provided
    let filename = params.filename.unwrap_or_else(|| {
        format!("screenshot_{}.jpg", chrono::Utc::now().timestamp_millis())
    });

    let filepath = temp_imgs_path.join(&filename);

    // Decode base64 image data
    // Handle both raw base64 and data URL format
    let image_data = if params.image_data.contains(",") {
        // Data URL format: data:image/jpeg;base64,/9j/4AAQ...
        params.image_data.split(",").nth(1).unwrap_or(&params.image_data)
    } else {
        &params.image_data
    };

    let decoded = BASE64
        .decode(image_data)
        .map_err(|e| Error::BadRequest(format!("Invalid base64 image data: {}", e)))?;

    // Write image to file
    tokio::fs::write(&filepath, decoded)
        .await
        .map_err(|e| Error::BadRequest(format!("Failed to write screenshot: {}", e)))?;

    // Return relative path from repo root
    let relative_path = format!("temp_imgs/{}", filename);

    tracing::info!("Saved screenshot to {}", filepath.display());

    format::json(SaveScreenshotResponse {
        filepath: relative_path,
        filename,
    })
}

/// Generate tasks by spawning the BA agent with transcript and screenshots
#[debug_handler]
async fn generate_tasks(
    State(ctx): State<AppContext>,
    Json(params): Json<GenerateTasksRequest>,
) -> Result<Response> {
    // Get settings
    let repo_path = get_setting(&ctx.db, "target_repo_path")
        .await
        .ok_or_else(|| Error::BadRequest("Target repo path not configured".to_string()))?;

    let ba_prompt = get_setting(&ctx.db, "ba_prompt")
        .await
        .unwrap_or_else(|| {
            "You are a Business Analyst. Analyze the user's requirements from their voice recording \
             and any screenshots provided. Create clear, actionable task descriptions that a \
             developer can understand and implement. Focus on breaking down the requirements into \
             discrete, well-defined tasks.".to_string()
        });

    // Build the prompt with transcript and screenshot references
    let mut full_prompt = format!(
        "## Business Analyst Instructions\n{}\n\n## User's Voice Transcription\n{}\n",
        ba_prompt, params.transcript
    );

    // Add screenshot references with @ prefix
    if !params.screenshots.is_empty() {
        full_prompt.push_str("\n## Screenshots for Context\n");
        full_prompt.push_str("The following screenshots were captured during the recording. Review them for visual context:\n\n");
        for screenshot in &params.screenshots {
            full_prompt.push_str(&format!("@{}\n", screenshot));
        }
    }

    full_prompt.push_str("\n## Your Task\n");
    full_prompt.push_str("Based on the transcription and screenshots above, create a summary of what the user wants to accomplish and suggest how to break this down into implementable tasks.");

    // Determine which agent command to use
    let (agent_name, agent_cmd) = match params.agent {
        AgentType::Claude => ("claude", "claude"),
        AgentType::Codex => ("codex", "codex"),
        AgentType::Gemini => ("gemini", "gemini"),
    };

    // Check if agent command is available
    let agent_check = Command::new("which")
        .arg(agent_cmd)
        .output()
        .await;

    if agent_check.is_err() || !agent_check.unwrap().status.success() {
        return Err(Error::BadRequest(
            format!("The '{}' command is not found in PATH. Please install it first.", agent_cmd)
        ));
    }

    // Spawn the agent using script for PTY
    // Claude: script -q /dev/null claude -p "prompt" --dangerously-skip-permissions
    // Codex: script -q /dev/null codex exec "prompt" --full-auto
    // Gemini: script -q /dev/null gemini "prompt" -y
    let child = match params.agent {
        AgentType::Claude => {
            Command::new("script")
                .arg("-q")
                .arg("/dev/null")
                .arg("claude")
                .arg("-p")
                .arg(&full_prompt)
                .arg("--dangerously-skip-permissions")
                .current_dir(&repo_path)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
        }
        AgentType::Codex => {
            Command::new("script")
                .arg("-q")
                .arg("/dev/null")
                .arg("codex")
                .arg("exec")
                .arg(&full_prompt)
                .arg("--full-auto")
                .current_dir(&repo_path)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
        }
        AgentType::Gemini => {
            Command::new("script")
                .arg("-q")
                .arg("/dev/null")
                .arg("gemini")
                .arg(&full_prompt)
                .arg("-y")
                .current_dir(&repo_path)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
        }
    };

    match child {
        Ok(process) => {
            let pid = process.id();
            let agent_name_owned = agent_name.to_string();

            // Spawn a task to wait for completion and log output
            tokio::spawn(async move {
                match process.wait_with_output().await {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        tracing::info!("{} agent completed. Exit code: {:?}", agent_name_owned, output.status.code());
                        if !stdout.is_empty() {
                            tracing::info!("{} agent stdout: {}", agent_name_owned, stdout);
                        }
                        if !stderr.is_empty() {
                            tracing::warn!("{} agent stderr: {}", agent_name_owned, stderr);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to wait for {} agent: {}", agent_name_owned, e);
                    }
                }
            });

            tracing::info!("Spawned {} agent with PID {:?}", agent_name, pid);

            format::json(GenerateTasksResponse {
                success: true,
                message: format!("{} agent spawned successfully (PID: {:?})", agent_name, pid),
                session_id: pid.map(|p| p.to_string()),
            })
        }
        Err(e) => {
            tracing::error!("Failed to spawn {} agent: {}", agent_name, e);
            Err(Error::BadRequest(format!("Failed to spawn {} agent: {}", agent_name, e)))
        }
    }
}

/// Clear all screenshots from temp_imgs folder
#[debug_handler]
async fn clear_screenshots(State(ctx): State<AppContext>) -> Result<Response> {
    let repo_path = get_setting(&ctx.db, "target_repo_path")
        .await
        .ok_or_else(|| Error::BadRequest("Target repo path not configured".to_string()))?;

    let temp_imgs_path = PathBuf::from(&repo_path).join("temp_imgs");

    if temp_imgs_path.exists() {
        // Remove all files in temp_imgs
        let mut entries = tokio::fs::read_dir(&temp_imgs_path)
            .await
            .map_err(|e| Error::BadRequest(format!("Failed to read temp_imgs: {}", e)))?;

        let mut count = 0;
        while let Some(entry) = entries.next_entry().await.map_err(|e| Error::BadRequest(e.to_string()))? {
            let path = entry.path();
            if path.is_file() {
                tokio::fs::remove_file(&path)
                    .await
                    .map_err(|e| Error::BadRequest(format!("Failed to delete {}: {}", path.display(), e)))?;
                count += 1;
            }
        }

        tracing::info!("Cleared {} screenshots from temp_imgs", count);

        format::json(serde_json::json!({
            "success": true,
            "message": format!("Cleared {} screenshots", count)
        }))
    } else {
        format::json(serde_json::json!({
            "success": true,
            "message": "temp_imgs folder does not exist"
        }))
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/voice")
        .add("/screenshot", post(save_screenshot))
        .add("/generate-tasks", post(generate_tasks))
        .add("/screenshots", axum::routing::delete(clear_screenshots))
}
