//! ClickUp hierarchy browser controller

use crate::services::clickup::ClickUpClient;
use loco_rs::prelude::*;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Get all workspaces (teams) the user has access to
#[debug_handler]
async fn get_workspaces() -> Result<Response> {
    let client = match ClickUpClient::from_env() {
        Ok(c) => c,
        Err(e) => {
            return format::json(ErrorResponse {
                error: e.to_string(),
            });
        }
    };

    match client.get_workspaces().await {
        Ok(teams) => format::json(teams),
        Err(e) => format::json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

/// Get all spaces in a workspace
#[debug_handler]
async fn get_spaces(Path(team_id): Path<String>) -> Result<Response> {
    let client = match ClickUpClient::from_env() {
        Ok(c) => c,
        Err(e) => {
            return format::json(ErrorResponse {
                error: e.to_string(),
            });
        }
    };

    match client.get_spaces(&team_id).await {
        Ok(spaces) => format::json(spaces),
        Err(e) => format::json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

/// Get all folders in a space
#[debug_handler]
async fn get_folders(Path(space_id): Path<String>) -> Result<Response> {
    let client = match ClickUpClient::from_env() {
        Ok(c) => c,
        Err(e) => {
            return format::json(ErrorResponse {
                error: e.to_string(),
            });
        }
    };

    match client.get_folders(&space_id).await {
        Ok(folders) => format::json(folders),
        Err(e) => format::json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

/// Get all lists in a folder
#[debug_handler]
async fn get_lists_in_folder(Path(folder_id): Path<String>) -> Result<Response> {
    let client = match ClickUpClient::from_env() {
        Ok(c) => c,
        Err(e) => {
            return format::json(ErrorResponse {
                error: e.to_string(),
            });
        }
    };

    match client.get_lists_in_folder(&folder_id).await {
        Ok(lists) => format::json(lists),
        Err(e) => format::json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

/// Get folderless lists in a space
#[debug_handler]
async fn get_folderless_lists(Path(space_id): Path<String>) -> Result<Response> {
    let client = match ClickUpClient::from_env() {
        Ok(c) => c,
        Err(e) => {
            return format::json(ErrorResponse {
                error: e.to_string(),
            });
        }
    };

    match client.get_folderless_lists(&space_id).await {
        Ok(lists) => format::json(lists),
        Err(e) => format::json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

/// Get statuses for a list
#[debug_handler]
async fn get_list_statuses(Path(list_id): Path<String>) -> Result<Response> {
    let client = match ClickUpClient::from_env() {
        Ok(c) => c,
        Err(e) => {
            return format::json(ErrorResponse {
                error: e.to_string(),
            });
        }
    };

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
