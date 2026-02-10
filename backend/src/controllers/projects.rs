//! Projects controller for managing multi-project support

use crate::models::_entities::projects;
use crate::services::project_git::{clone_repo, validate_repo};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::path::Path as FsPath;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub repo_path: String,
    pub github_url: Option<String>,
    pub dev_branch: String,
    pub clickup_workspace_id: Option<String>,
    pub clickup_space_id: Option<String>,
    pub clickup_folder_id: Option<String>,
    pub clickup_list_id: Option<String>,
    pub clickup_api_key: Option<String>,
    pub agent_prompt: Option<String>,
    pub agent_model: String,
    pub parallel_limit: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct ProjectListItem {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub repo_path: String,
    pub dev_branch: String,
    pub workflow_count: i64,
    pub active_task_count: i64,
}

#[derive(Debug, Serialize)]
pub struct FolderInfo {
    pub name: String,
    pub path: String,
    pub is_git_repo: bool,
}

#[derive(Debug, Serialize)]
pub struct FolderListResponse {
    pub current_path: String,
    pub base_path: String,
    pub folders: Vec<FolderInfo>,
    pub can_go_up: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub repo_path: String,
    pub dev_branch: Option<String>,
    pub clickup_api_key: Option<String>,
    pub clickup_workspace_id: Option<String>,
    pub clickup_space_id: Option<String>,
    pub clickup_folder_id: Option<String>,
    pub clickup_list_id: Option<String>,
    pub agent_prompt: Option<String>,
    pub agent_model: Option<String>,
    pub parallel_limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub repo_path: Option<String>,
    pub dev_branch: Option<String>,
    pub clickup_api_key: Option<String>,
    pub clickup_workspace_id: Option<String>,
    pub clickup_space_id: Option<String>,
    pub clickup_folder_id: Option<String>,
    pub clickup_list_id: Option<String>,
    pub agent_prompt: Option<String>,
    pub agent_model: Option<String>,
    pub parallel_limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CloneProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub github_url: String,
    pub target_path: String,
    pub dev_branch: Option<String>,
    pub clickup_api_key: Option<String>,
    pub clickup_workspace_id: Option<String>,
    pub clickup_space_id: Option<String>,
    pub clickup_folder_id: Option<String>,
    pub clickup_list_id: Option<String>,
    pub agent_prompt: Option<String>,
    pub agent_model: Option<String>,
    pub parallel_limit: Option<i32>,
}

impl From<projects::Model> for ProjectResponse {
    fn from(model: projects::Model) -> Self {
        ProjectResponse {
            id: model.id,
            name: model.name,
            description: model.description,
            status: model.status,
            repo_path: model.repo_path,
            github_url: model.github_url,
            dev_branch: model.dev_branch,
            clickup_workspace_id: model.clickup_workspace_id,
            clickup_space_id: model.clickup_space_id,
            clickup_folder_id: model.clickup_folder_id,
            clickup_list_id: model.clickup_list_id,
            clickup_api_key: model.clickup_api_key,
            agent_prompt: model.agent_prompt,
            agent_model: model.agent_model,
            parallel_limit: model.parallel_limit,
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
        }
    }
}

/// List all projects with basic stats
#[debug_handler]
async fn list_projects(State(ctx): State<AppContext>) -> Result<Response> {
    let projects_list = projects::Entity::find()
        .all(&ctx.db)
        .await?;

    let mut response = Vec::new();

    for project in projects_list {
        // For now, simplified stats (can be enhanced later with actual queries)
        response.push(ProjectListItem {
            id: project.id,
            name: project.name,
            description: project.description,
            status: project.status,
            repo_path: project.repo_path,
            dev_branch: project.dev_branch,
            workflow_count: 0,  // TODO: Query workflow count for project
            active_task_count: 0, // TODO: Query active task count for project
        });
    }

    format::json(response)
}

/// Create a new project
#[debug_handler]
async fn create_project(
    State(ctx): State<AppContext>,
    Json(params): Json<CreateProjectRequest>,
) -> Result<Response> {
    // Validate repo path
    let repo_path = params.repo_path.trim();
    if repo_path.is_empty() {
        return Err(Error::BadRequest("repo_path cannot be empty".to_string()));
    }

    // Verify the repo path is valid
    if !FsPath::new(repo_path).exists() {
        return Err(Error::BadRequest(format!("Repository path does not exist: {}", repo_path)));
    }

    let now = chrono::Utc::now();
    let dev_branch = params.dev_branch.unwrap_or_else(|| "dev".to_string());
    let agent_model = params.agent_model.unwrap_or_else(|| "claude".to_string());
    let parallel_limit = params.parallel_limit.unwrap_or(1);

    let project = projects::ActiveModel {
        name: Set(params.name.trim().to_string()),
        description: Set(params.description.map(|d| d.trim().to_string())),
        status: Set("active".to_string()),
        repo_path: Set(repo_path.to_string()),
        github_url: Set(None),
        dev_branch: Set(dev_branch),
        clickup_api_key: Set(params.clickup_api_key),
        clickup_workspace_id: Set(params.clickup_workspace_id),
        clickup_space_id: Set(params.clickup_space_id),
        clickup_folder_id: Set(params.clickup_folder_id),
        clickup_list_id: Set(params.clickup_list_id),
        agent_prompt: Set(params.agent_prompt),
        agent_model: Set(agent_model),
        parallel_limit: Set(parallel_limit),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };

    let inserted = project.insert(&ctx.db).await?;
    format::json(ProjectResponse::from(inserted))
}

/// Get a specific project by ID
#[debug_handler]
async fn get_project(State(ctx): State<AppContext>, Path(id): Path<i32>) -> Result<Response> {
    let project = projects::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    format::json(ProjectResponse::from(project))
}

/// Update a project
#[debug_handler]
async fn update_project(
    State(ctx): State<AppContext>,
    Path(id): Path<i32>,
    Json(params): Json<UpdateProjectRequest>,
) -> Result<Response> {
    let project = projects::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let mut active: projects::ActiveModel = project.into();

    if let Some(name) = params.name {
        active.name = Set(name.trim().to_string());
    }

    if let Some(description) = params.description {
        active.description = Set(Some(description.trim().to_string()));
    }

    if let Some(repo_path) = params.repo_path {
        if !FsPath::new(&repo_path).exists() {
            return Err(Error::BadRequest(format!("Repository path does not exist: {}", repo_path)));
        }
        active.repo_path = Set(repo_path);
    }

    if let Some(dev_branch) = params.dev_branch {
        active.dev_branch = Set(dev_branch);
    }

    if let Some(api_key) = params.clickup_api_key {
        active.clickup_api_key = Set(Some(api_key));
    }

    if let Some(workspace_id) = params.clickup_workspace_id {
        active.clickup_workspace_id = Set(Some(workspace_id));
    }

    if let Some(space_id) = params.clickup_space_id {
        active.clickup_space_id = Set(Some(space_id));
    }

    if let Some(folder_id) = params.clickup_folder_id {
        active.clickup_folder_id = Set(Some(folder_id));
    }

    if let Some(list_id) = params.clickup_list_id {
        active.clickup_list_id = Set(Some(list_id));
    }

    if let Some(prompt) = params.agent_prompt {
        active.agent_prompt = Set(Some(prompt));
    }

    if let Some(model) = params.agent_model {
        active.agent_model = Set(model);
    }

    if let Some(limit) = params.parallel_limit {
        active.parallel_limit = Set(limit);
    }

    active.updated_at = Set(chrono::Utc::now().into());
    let updated = active.update(&ctx.db).await?;

    format::json(ProjectResponse::from(updated))
}

/// Delete a project
#[debug_handler]
async fn delete_project(State(ctx): State<AppContext>, Path(id): Path<i32>) -> Result<Response> {
    let project = projects::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    // Delete associated workflows and tasks first (cascade)
    crate::models::_entities::workflow_configs::Entity::delete_many()
        .filter(crate::models::_entities::workflow_configs::Column::ProjectId.eq(id))
        .exec(&ctx.db)
        .await?;

    crate::models::_entities::orchestrator_tasks::Entity::delete_many()
        .filter(crate::models::_entities::orchestrator_tasks::Column::ProjectId.eq(id))
        .exec(&ctx.db)
        .await?;

    // Delete the project
    let active: projects::ActiveModel = project.into();
    active.delete(&ctx.db).await?;

    format::empty()
}

/// Archive a project
#[debug_handler]
async fn archive_project(
    State(ctx): State<AppContext>,
    Path(id): Path<i32>,
) -> Result<Response> {
    let project = projects::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    let mut active: projects::ActiveModel = project.into();
    active.status = Set("archived".to_string());
    active.updated_at = Set(chrono::Utc::now().into());
    let updated = active.update(&ctx.db).await?;

    format::json(ProjectResponse::from(updated))
}

/// Clone a project from GitHub
#[debug_handler]
async fn clone_project(
    State(ctx): State<AppContext>,
    Json(params): Json<CloneProjectRequest>,
) -> Result<Response> {
    let target_path = params.target_path.trim();
    if target_path.is_empty() {
        return Err(Error::BadRequest("target_path cannot be empty".to_string()));
    }

    let github_url = params.github_url.trim();
    if github_url.is_empty() {
        return Err(Error::BadRequest("github_url cannot be empty".to_string()));
    }

    // Clone the repository
    clone_repo(github_url, target_path)
        .await
        .map_err(|e| Error::BadRequest(format!("Failed to clone repository: {}", e)))?;

    // Verify it's a valid git repo
    validate_repo(target_path)
        .await
        .map_err(|e| Error::BadRequest(format!("Invalid repository: {}", e)))?;

    // Create project record
    let now = chrono::Utc::now();
    let dev_branch = params.dev_branch.unwrap_or_else(|| "dev".to_string());
    let agent_model = params.agent_model.unwrap_or_else(|| "claude".to_string());
    let parallel_limit = params.parallel_limit.unwrap_or(1);

    let project = projects::ActiveModel {
        name: Set(params.name.trim().to_string()),
        description: Set(params.description.map(|d| d.trim().to_string())),
        status: Set("active".to_string()),
        repo_path: Set(target_path.to_string()),
        github_url: Set(Some(github_url.to_string())),
        dev_branch: Set(dev_branch),
        clickup_api_key: Set(params.clickup_api_key),
        clickup_workspace_id: Set(params.clickup_workspace_id),
        clickup_space_id: Set(params.clickup_space_id),
        clickup_folder_id: Set(params.clickup_folder_id),
        clickup_list_id: Set(params.clickup_list_id),
        agent_prompt: Set(params.agent_prompt),
        agent_model: Set(agent_model),
        parallel_limit: Set(parallel_limit),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };

    let inserted = project.insert(&ctx.db).await?;
    format::json(ProjectResponse::from(inserted))
}

/// List available folders for project creation
#[debug_handler]
async fn list_folders(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Response> {
    // Get base path from env or use home directory
    let base_path_str = std::env::var("PROJECTS_BASE_PATH")
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "/home".to_string())
        });

    let base_path = PathBuf::from(&base_path_str);

    // Get requested path from query params
    let requested_path = params.get("path")
        .map(|p| p.as_str())
        .unwrap_or("");

    let mut current_path = if requested_path.is_empty() {
        base_path.clone()
    } else {
        let req_path = PathBuf::from(requested_path);
        // Security: ensure the path is within base_path
        if !req_path.starts_with(&base_path) {
            return Err(Error::BadRequest("Path is outside allowed base directory".to_string()));
        }
        req_path
    };

    // Verify the current path exists and is a directory
    if !current_path.exists() || !current_path.is_dir() {
        current_path = base_path.clone();
    }

    let mut folders = Vec::new();

    // List directories in current path
    if let Ok(entries) = std::fs::read_dir(&current_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip hidden directories
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy().to_string();
                    if !name_str.starts_with('.') {
                        let is_git_repo = path.join(".git").exists();
                        folders.push(FolderInfo {
                            name: name_str,
                            path: path.to_string_lossy().to_string(),
                            is_git_repo,
                        });
                    }
                }
            }
        }
    }

    // Sort alphabetically
    folders.sort_by(|a, b| a.name.cmp(&b.name));

    // Check if we can go up
    let can_go_up = current_path.parent().map_or(false, |parent| {
        parent.starts_with(&base_path)
    });

    format::json(FolderListResponse {
        current_path: current_path.to_string_lossy().to_string(),
        base_path: base_path.to_string_lossy().to_string(),
        folders,
        can_go_up,
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/projects")
        .add("/", get(list_projects))
        .add("/", post(create_project))
        .add("/folders", get(list_folders))
        .add("/{id}", get(get_project))
        .add("/{id}", put(update_project))
        .add("/{id}", delete(delete_project))
        .add("/{id}/archive", post(archive_project))
        .add("/clone", post(clone_project))
}
