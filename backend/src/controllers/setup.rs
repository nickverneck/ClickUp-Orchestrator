//! Setup controller for first-time configuration

use crate::models::_entities::settings;
use crate::services::clickup::ClickUpClient;
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct SetupStatus {
    pub is_complete: bool,
    pub has_api_key: bool,
    pub has_list_selected: bool,
    pub has_repo_configured: bool,
    pub api_key_valid: bool,
}

#[derive(Debug, Deserialize)]
pub struct SaveApiKeyRequest {
    pub api_key: String,
}

#[derive(Debug, Serialize)]
pub struct SaveApiKeyResponse {
    pub success: bool,
    pub valid: bool,
    pub error: Option<String>,
}

/// Get setup status
#[debug_handler]
async fn get_status(State(ctx): State<AppContext>) -> Result<Response> {
    // Check if API key exists
    let has_api_key = std::env::var("CLICKUP_API_KEY")
        .map(|k| !k.is_empty())
        .unwrap_or(false);

    // Check if API key is valid by trying to fetch workspaces
    let api_key_valid = if has_api_key {
        match ClickUpClient::from_env() {
            Ok(client) => client.get_workspaces().await.is_ok(),
            Err(_) => false,
        }
    } else {
        false
    };

    // Check if list is selected
    let has_list_selected = settings::Entity::find()
        .filter(settings::Column::Key.eq("clickup_list_id"))
        .one(&ctx.db)
        .await
        .ok()
        .flatten()
        .map(|s| !s.value.is_empty())
        .unwrap_or(false);

    // Check if repo is configured
    let has_repo_configured = settings::Entity::find()
        .filter(settings::Column::Key.eq("target_repo_path"))
        .one(&ctx.db)
        .await
        .ok()
        .flatten()
        .map(|s| !s.value.is_empty())
        .unwrap_or(false);

    let is_complete = has_api_key && api_key_valid && has_list_selected;

    format::json(SetupStatus {
        is_complete,
        has_api_key,
        has_list_selected,
        has_repo_configured,
        api_key_valid,
    })
}

/// Save API key to .env file
#[debug_handler]
async fn save_api_key(Json(params): Json<SaveApiKeyRequest>) -> Result<Response> {
    let api_key = params.api_key.trim();

    if api_key.is_empty() {
        return format::json(SaveApiKeyResponse {
            success: false,
            valid: false,
            error: Some("API key cannot be empty".to_string()),
        });
    }

    // Validate API key by trying to fetch workspaces
    let client = ClickUpClient::new(api_key.to_string());
    let valid = match client.get_workspaces().await {
        Ok(teams) => !teams.is_empty(),
        Err(e) => {
            return format::json(SaveApiKeyResponse {
                success: false,
                valid: false,
                error: Some(format!("Invalid API key: {}", e)),
            });
        }
    };

    if !valid {
        return format::json(SaveApiKeyResponse {
            success: false,
            valid: false,
            error: Some("API key is valid but no workspaces found".to_string()),
        });
    }

    // Save to .env file in parent directory
    let env_path = Path::new("../.env");
    let env_content = format!("CLICKUP_API_KEY={}\n", api_key);

    // Read existing .env content if it exists
    let existing_content = fs::read_to_string(env_path).unwrap_or_default();

    // Check if CLICKUP_API_KEY already exists
    let new_content = if existing_content.contains("CLICKUP_API_KEY") {
        // Replace existing key
        existing_content
            .lines()
            .map(|line| {
                if line.starts_with("CLICKUP_API_KEY") {
                    format!("CLICKUP_API_KEY={}", api_key)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    } else {
        // Append new key
        if existing_content.is_empty() {
            env_content
        } else {
            format!("{}\n{}", existing_content.trim_end(), env_content.trim())
        }
    };

    // Write .env file
    if let Err(e) = fs::write(env_path, &new_content) {
        return format::json(SaveApiKeyResponse {
            success: false,
            valid: true,
            error: Some(format!("Failed to save .env file: {}", e)),
        });
    }

    // Also set the environment variable for this process
    std::env::set_var("CLICKUP_API_KEY", api_key);

    format::json(SaveApiKeyResponse {
        success: true,
        valid: true,
        error: None,
    })
}

/// Mark setup as complete
#[debug_handler]
async fn complete_setup(State(ctx): State<AppContext>) -> Result<Response> {
    // Verify everything is configured
    let has_api_key = std::env::var("CLICKUP_API_KEY")
        .map(|k| !k.is_empty())
        .unwrap_or(false);

    let has_list_selected = settings::Entity::find()
        .filter(settings::Column::Key.eq("clickup_list_id"))
        .one(&ctx.db)
        .await
        .ok()
        .flatten()
        .map(|s| !s.value.is_empty())
        .unwrap_or(false);

    if !has_api_key || !has_list_selected {
        return format::json(serde_json::json!({
            "success": false,
            "error": "Setup is not complete. Please configure API key and select a list."
        }));
    }

    format::json(serde_json::json!({
        "success": true
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/setup")
        .add("/status", get(get_status))
        .add("/api-key", post(save_api_key))
        .add("/complete", post(complete_setup))
}
