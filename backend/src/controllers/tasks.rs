//! Tasks controller for managing orchestrator tasks

use crate::models::_entities::{orchestrator_tasks, process_sessions, settings};
use crate::services::process_manager::PROCESS_MANAGER;
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: i32,
    pub clickup_task_id: String,
    pub name: String,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub status: String,
    pub worktree_path: Option<String>,
    pub time_spent_ms: i32,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub is_running: bool,
}

impl From<orchestrator_tasks::Model> for TaskResponse {
    fn from(task: orchestrator_tasks::Model) -> Self {
        let is_running = PROCESS_MANAGER.is_running(task.id);
        let now = chrono::Utc::now();
        let time_spent_ms = compute_time_spent_ms(&task, is_running, now);
        Self {
            id: task.id,
            clickup_task_id: task.clickup_task_id,
            name: task.name,
            description: task.description,
            priority: task.priority,
            status: task.status,
            worktree_path: task.worktree_path,
            time_spent_ms,
            started_at: task.started_at.map(|t| t.to_rfc3339()),
            completed_at: task.completed_at.map(|t| t.to_rfc3339()),
            is_running,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
}

fn sanitize_worktree_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .to_lowercase()
}

fn elapsed_ms_since(
    started_at: &chrono::DateTime<chrono::FixedOffset>,
    now: chrono::DateTime<chrono::Utc>,
) -> i32 {
    let elapsed_ms = now
        .signed_duration_since(started_at.with_timezone(&chrono::Utc))
        .num_milliseconds();
    if elapsed_ms <= 0 {
        0
    } else if elapsed_ms > i32::MAX as i64 {
        i32::MAX
    } else {
        elapsed_ms as i32
    }
}

fn compute_time_spent_ms(
    task: &orchestrator_tasks::Model,
    is_running: bool,
    now: chrono::DateTime<chrono::Utc>,
) -> i32 {
    let mut time_spent_ms = task.time_spent_ms;
    if is_running || task.status == "in_progress" {
        if let Some(started_at) = &task.started_at {
            let elapsed_ms = elapsed_ms_since(started_at, now);
            time_spent_ms = time_spent_ms.saturating_add(elapsed_ms);
        }
    }
    time_spent_ms
}

/// List all tasks
#[debug_handler]
async fn list(State(ctx): State<AppContext>, Query(query): Query<ListQuery>) -> Result<Response> {
    let mut find = orchestrator_tasks::Entity::find()
        .order_by_desc(orchestrator_tasks::Column::CreatedAt);

    if let Some(status) = &query.status {
        find = find.filter(orchestrator_tasks::Column::Status.eq(status));
    }

    let tasks: Vec<TaskResponse> = find
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(TaskResponse::from)
        .collect();

    format::json(tasks)
}

/// Get a single task by ID
#[debug_handler]
async fn get_one(State(ctx): State<AppContext>, Path(id): Path<i32>) -> Result<Response> {
    let task = orchestrator_tasks::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    format::json(TaskResponse::from(task))
}

/// Stop a running task
#[debug_handler]
async fn stop(State(ctx): State<AppContext>, Path(id): Path<i32>) -> Result<Response> {
    let task = orchestrator_tasks::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    if task.status != "in_progress" {
        return Err(Error::BadRequest("Task is not in progress".to_string()));
    }

    // Kill the process
    if let Err(e) = PROCESS_MANAGER.kill_process(id).await {
        tracing::error!("Failed to kill process: {}", e);
    }

    let now = chrono::Utc::now();
    let time_spent_ms = match task.started_at.as_ref() {
        Some(started_at) => task
            .time_spent_ms
            .saturating_add(elapsed_ms_since(started_at, now)),
        None => task.time_spent_ms,
    };

    // Update task status
    let mut active: orchestrator_tasks::ActiveModel = task.into();
    active.status = Set("stopped".to_string());
    active.time_spent_ms = Set(time_spent_ms);
    active.updated_at = Set(now.into());
    let updated = active.update(&ctx.db).await?;

    // Update process session
    let _ = process_sessions::Entity::update_many()
        .filter(process_sessions::Column::TaskId.eq(id))
        .filter(process_sessions::Column::EndedAt.is_null())
        .col_expr(
            process_sessions::Column::EndedAt,
            sea_orm::sea_query::Expr::value(now),
        )
        .exec(&ctx.db)
        .await;

    format::json(TaskResponse::from(updated))
}

/// Restart a stopped task
#[debug_handler]
async fn restart(State(ctx): State<AppContext>, Path(id): Path<i32>) -> Result<Response> {
    let task = orchestrator_tasks::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    if task.status != "stopped" && task.status != "failed" {
        return Err(Error::BadRequest(
            "Task must be stopped or failed to restart".to_string(),
        ));
    }

    let raw_worktree_path = task.worktree_path.clone().ok_or(Error::BadRequest(
        "Task has no worktree path".to_string(),
    ))?;
    let mut worktree_path = raw_worktree_path.trim().to_string();
    let mut worktree_path_needs_update = worktree_path != raw_worktree_path;

    // Check if worktree exists, if not try to resolve it using current settings
    if !Path::new(&worktree_path).exists() {
        if let Some(target_repo_path) = settings::Entity::find()
            .filter(settings::Column::Key.eq("target_repo_path"))
            .one(&ctx.db)
            .await
            .ok()
            .flatten()
            .map(|s| s.value)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
        {
            let worktree_name = Path::new(&worktree_path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.to_string())
                .unwrap_or_else(|| sanitize_worktree_name(&task.name));
            let candidate_path = format!("{}/worktrees/{}", target_repo_path, worktree_name);
            if Path::new(&candidate_path).exists() {
                worktree_path = candidate_path;
                worktree_path_needs_update = true;
            }
        }
    }

    if !Path::new(&worktree_path).exists() {
        tracing::warn!(
            "Worktree path does not exist: {}, will need to recreate",
            worktree_path
        );
        return Err(Error::BadRequest(format!(
            "Worktree path does not exist: {}. The task needs to be recreated.",
            worktree_path
        )));
    }

    // Get agent prompt from settings
    let agent_prompt = settings::Entity::find()
        .filter(settings::Column::Key.eq("agent_prompt"))
        .one(&ctx.db)
        .await
        .ok()
        .flatten()
        .map(|s| s.value)
        .filter(|v| !v.is_empty());

    // Build prompt from task description combined with agent prompt
    let task_description = task
        .description
        .clone()
        .unwrap_or_else(|| format!("Complete task: {}", task.name));

    let prompt = match &agent_prompt {
        Some(global_prompt) if !global_prompt.is_empty() => {
            format!(
                "## Task\n{}\n\n## Instructions\n{}",
                task_description, global_prompt
            )
        }
        _ => task_description,
    };

    // Spawn new process
    match PROCESS_MANAGER
        .spawn_agent(id, &prompt, &worktree_path)
        .await
    {
        Ok(pid) => {
            tracing::info!("Restarted task {} with PID {}", id, pid);

            // Update task status
            let mut active: orchestrator_tasks::ActiveModel = task.into();
            active.status = Set("in_progress".to_string());
            active.started_at = Set(Some(chrono::Utc::now().into()));
            active.updated_at = Set(chrono::Utc::now().into());
            if worktree_path_needs_update {
                active.worktree_path = Set(Some(worktree_path.clone()));
            }
            let updated = active.update(&ctx.db).await?;

            // Create new process session
            let session = process_sessions::ActiveModel {
                task_id: Set(id),
                pid: Set(Some(pid as i32)),
                started_at: Set(chrono::Utc::now().into()),
                ended_at: Set(None),
                exit_code: Set(None),
                created_at: Set(chrono::Utc::now().into()),
                updated_at: Set(chrono::Utc::now().into()),
                ..Default::default()
            };
            let _ = process_sessions::Entity::insert(session).exec(&ctx.db).await;

            format::json(TaskResponse::from(updated))
        }
        Err(e) => {
            tracing::error!("Failed to restart task {}: {}", id, e);
            Err(Error::BadRequest(format!("Failed to restart: {}", e)))
        }
    }
}

/// Delete a task
#[debug_handler]
async fn delete(State(ctx): State<AppContext>, Path(id): Path<i32>) -> Result<Response> {
    let task = orchestrator_tasks::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    // If task is in progress, kill the process first
    if task.status == "in_progress" {
        if let Err(e) = PROCESS_MANAGER.kill_process(id).await {
            tracing::warn!("Failed to kill process for task {}: {}", id, e);
        }
    }

    // Delete associated process sessions
    process_sessions::Entity::delete_many()
        .filter(process_sessions::Column::TaskId.eq(id))
        .exec(&ctx.db)
        .await?;

    // Delete the task
    orchestrator_tasks::Entity::delete_by_id(id)
        .exec(&ctx.db)
        .await?;

    // Optionally clean up worktree (don't fail if it doesn't work)
    if let Some(worktree_path) = &task.worktree_path {
        if std::path::Path::new(worktree_path).exists() {
            // Try to remove the worktree
            let _ = tokio::process::Command::new("git")
                .args(["worktree", "remove", "--force", worktree_path])
                .output()
                .await;
        }
    }

    tracing::info!("Deleted task {} ({})", id, task.name);

    format::json(serde_json::json!({
        "success": true,
        "message": format!("Task {} deleted", id)
    }))
}

/// Get task stats
#[debug_handler]
async fn stats(State(ctx): State<AppContext>) -> Result<Response> {
    let queued = orchestrator_tasks::Entity::find()
        .filter(orchestrator_tasks::Column::Status.eq("queued"))
        .count(&ctx.db)
        .await?;

    let in_progress = orchestrator_tasks::Entity::find()
        .filter(orchestrator_tasks::Column::Status.eq("in_progress"))
        .count(&ctx.db)
        .await?;

    let stopped = orchestrator_tasks::Entity::find()
        .filter(orchestrator_tasks::Column::Status.eq("stopped"))
        .count(&ctx.db)
        .await?;

    let completed = orchestrator_tasks::Entity::find()
        .filter(orchestrator_tasks::Column::Status.eq("completed"))
        .count(&ctx.db)
        .await?;

    let failed = orchestrator_tasks::Entity::find()
        .filter(orchestrator_tasks::Column::Status.eq("failed"))
        .count(&ctx.db)
        .await?;

    format::json(serde_json::json!({
        "queued": queued,
        "in_progress": in_progress,
        "stopped": stopped,
        "completed": completed,
        "failed": failed,
        "running_processes": PROCESS_MANAGER.running_tasks().len()
    }))
}

/// Get task logs
#[debug_handler]
async fn get_logs(State(ctx): State<AppContext>, Path(id): Path<i32>) -> Result<Response> {
    let task = orchestrator_tasks::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    format::json(serde_json::json!({
        "task_id": task.id,
        "name": task.name,
        "status": task.status,
        "log": task.output_log
    }))
}

/// Manually mark a task as completed (for stuck tasks)
#[debug_handler]
async fn mark_complete(State(ctx): State<AppContext>, Path(id): Path<i32>) -> Result<Response> {
    let task = orchestrator_tasks::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    // Kill any running process for this task
    if PROCESS_MANAGER.is_running(id) {
        let _ = PROCESS_MANAGER.kill_process(id).await;
    }

    let now = chrono::Utc::now();

    let time_spent_ms = match task.started_at.as_ref() {
        Some(started_at) if task.status == "in_progress" => task
            .time_spent_ms
            .saturating_add(elapsed_ms_since(started_at, now)),
        _ => task.time_spent_ms,
    };

    // Update task status to completed
    let mut active: orchestrator_tasks::ActiveModel = task.into();
    active.status = Set("completed".to_string());
    active.completed_at = Set(Some(now.into()));
    active.time_spent_ms = Set(time_spent_ms);
    active.updated_at = Set(now.into());
    let updated = active.update(&ctx.db).await?;

    // Update any open process sessions
    let _ = process_sessions::Entity::update_many()
        .filter(process_sessions::Column::TaskId.eq(id))
        .filter(process_sessions::Column::EndedAt.is_null())
        .col_expr(
            process_sessions::Column::EndedAt,
            sea_orm::sea_query::Expr::value(now),
        )
        .col_expr(
            process_sessions::Column::ExitCode,
            sea_orm::sea_query::Expr::value(0),
        )
        .exec(&ctx.db)
        .await;

    tracing::info!("Task {} manually marked as completed", id);

    format::json(TaskResponse::from(updated))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/tasks")
        .add("/", get(list))
        .add("/stats", get(stats))
        .add("/{id}", get(get_one))
        .add("/{id}", axum::routing::delete(delete))
        .add("/{id}/stop", post(stop))
        .add("/{id}/restart", post(restart))
        .add("/{id}/complete", post(mark_complete))
        .add("/{id}/logs", get(get_logs))
}
