//! ClickUp hierarchy browser controller

use crate::services::clickup::ClickUpClient;
use axum::extract::Query;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiKeyQuery {
    pub api_key: Option<String>,
}

/// Get all workspaces (teams) the user has access to
#[debug_handler]
async fn get_workspaces(Query(params): Query<ApiKeyQuery>) -> Result<Response> {
    let api_key = match params.api_key {
        Some(key) if !key.is_empty() => key,
        _ => match std::env::var("CLICKUP_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                return format::json(ErrorResponse {
                    error: "API key not provided and not configured in environment".to_string(),
                });
            }
        },
    };

    let client = ClickUpClient::new(api_key);

    match client.get_workspaces().await {
        Ok(teams) => format::json(teams),
        Err(e) => format::json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

/// Get all spaces in a workspace
#[debug_handler]
async fn get_spaces(
    Path(team_id): Path<String>,
    Query(params): Query<ApiKeyQuery>,
) -> Result<Response> {
    let api_key = match params.api_key {
        Some(key) if !key.is_empty() => key,
        _ => match std::env::var("CLICKUP_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                return format::json(ErrorResponse {
                    error: "API key not provided and not configured in environment".to_string(),
                });
            }
        },
    };

    let client = ClickUpClient::new(api_key);

    match client.get_spaces(&team_id).await {
        Ok(spaces) => format::json(spaces),
        Err(e) => format::json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

/// Get all folders in a space
#[debug_handler]
async fn get_folders(
    Path(space_id): Path<String>,
    Query(params): Query<ApiKeyQuery>,
) -> Result<Response> {
    let api_key = match params.api_key {
        Some(key) if !key.is_empty() => key,
        _ => match std::env::var("CLICKUP_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                return format::json(ErrorResponse {
                    error: "API key not provided and not configured in environment".to_string(),
                });
            }
        },
    };

    let client = ClickUpClient::new(api_key);

    match client.get_folders(&space_id).await {
        Ok(folders) => format::json(folders),
        Err(e) => format::json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

/// Get all lists in a folder
#[debug_handler]
async fn get_lists_in_folder(
    Path(folder_id): Path<String>,
    Query(params): Query<ApiKeyQuery>,
) -> Result<Response> {
    let api_key = match params.api_key {
        Some(key) if !key.is_empty() => key,
        _ => match std::env::var("CLICKUP_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                return format::json(ErrorResponse {
                    error: "API key not provided and not configured in environment".to_string(),
                });
            }
        },
    };

    let client = ClickUpClient::new(api_key);

    match client.get_lists_in_folder(&folder_id).await {
        Ok(lists) => format::json(lists),
        Err(e) => format::json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

/// Get folderless lists in a space
#[debug_handler]
async fn get_folderless_lists(
    Path(space_id): Path<String>,
    Query(params): Query<ApiKeyQuery>,
) -> Result<Response> {
    let api_key = match params.api_key {
        Some(key) if !key.is_empty() => key,
        _ => match std::env::var("CLICKUP_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                return format::json(ErrorResponse {
                    error: "API key not provided and not configured in environment".to_string(),
                });
            }
        },
    };

    let client = ClickUpClient::new(api_key);

    match client.get_folderless_lists(&space_id).await {
        Ok(lists) => format::json(lists),
        Err(e) => format::json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

/// Get statuses for a list
#[debug_handler]
async fn get_list_statuses(
    Path(list_id): Path<String>,
    Query(params): Query<ApiKeyQuery>,
) -> Result<Response> {
    let api_key = match params.api_key {
        Some(key) if !key.is_empty() => key,
        _ => match std::env::var("CLICKUP_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                return format::json(ErrorResponse {
                    error: "API key not provided and not configured in environment".to_string(),
                });
            }
        },
    };

    let client = ClickUpClient::new(api_key);

    match client.get_list_statuses(&list_id).await {
        Ok(statuses) => format::json(statuses),
        Err(e) => format::json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/clickup")
        .add("/workspaces", get(get_workspaces))
        .add("/workspaces/{team_id}/spaces", get(get_spaces))
        .add("/spaces/{space_id}/folders", get(get_folders))
        .add("/folders/{folder_id}/lists", get(get_lists_in_folder))
        .add("/spaces/{space_id}/lists", get(get_folderless_lists))
        .add("/lists/{list_id}/statuses", get(get_list_statuses))
}
