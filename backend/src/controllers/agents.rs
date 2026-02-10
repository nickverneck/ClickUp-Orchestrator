//! Agents controller for managing CLI agent configurations

use loco_rs::prelude::*;
use serde::Serialize;
use tokio::process::Command;

#[derive(Debug, Serialize)]
pub struct AgentModel {
    pub id: String,
    pub provider: String,
    pub name: String,
}

/// List available OpenCode models by running `opencode models`
#[debug_handler]
async fn list_opencode_models() -> Result<Response> {
    // Check if opencode is available
    let which_check = Command::new("which")
        .arg("opencode")
        .output()
        .await;

    if which_check.is_err() || !which_check.unwrap().status.success() {
        return Err(Error::BadRequest(
            "The 'opencode' command is not found in PATH. Please install OpenCode.".to_string(),
        ));
    }

    let output = Command::new("opencode")
        .arg("models")
        .output()
        .await
        .map_err(|e| Error::BadRequest(format!("Failed to run 'opencode models': {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::BadRequest(format!(
            "opencode models failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let models: Vec<AgentModel> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let id = line.trim().to_string();
            let parts: Vec<&str> = id.splitn(2, '/').collect();
            let (provider, name) = if parts.len() == 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                ("unknown".to_string(), id.clone())
            };
            AgentModel { id, provider, name }
        })
        .collect();

    format::json(models)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/agents")
        .add("/opencode/models", get(list_opencode_models))
}
