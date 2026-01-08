//! ClickUp Poller Initializer
//!
//! Starts a background task that polls ClickUp for new tasks and processes them.

use async_trait::async_trait;
use axum::Router;
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use std::time::Duration;
use tokio::time::interval;

use crate::models::_entities::{orchestrator_tasks, settings};
use crate::services::clickup::{priority_to_int, ClickUpClient};
use crate::services::process_manager::PROCESS_MANAGER;
use crate::services::task_logs::{
    log_task_event, log_task_status_change, EVENT_CLICKUP, EVENT_SYSTEM,
};

pub struct ClickUpPollerInitializer;

impl ClickUpPollerInitializer {
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

    async fn poll_and_process(ctx: AppContext) {
        let db = &ctx.db;

        // Get settings
        let Some(list_id) = Self::get_setting(db, "clickup_list_id").await else {
            tracing::debug!("No ClickUp list configured, skipping poll");
            return;
        };

        let trigger_status = Self::get_setting(db, "trigger_status")
            .await
            .unwrap_or_else(|| "Ready for Dev".to_string());

        let target_status = Self::get_setting(db, "target_status")
            .await
            .unwrap_or_else(|| "In Development".to_string());

        let parallel_limit: usize = Self::get_setting(db, "parallel_limit")
            .await
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        let target_repo_path = match Self::get_setting(db, "target_repo_path")
            .await
            .map(|p| p.trim().to_string())
        {
            Some(p) if !p.is_empty() => p,
            _ => {
                tracing::debug!("No target repo path configured, skipping poll");
                return;
            }
        };

        let dev_branch = Self::get_setting(db, "dev_branch")
            .await
            .unwrap_or_else(|| "dev".to_string());

        // Get agent prompt (global instructions to combine with task description)
        let agent_prompt = Self::get_setting(db, "agent_prompt").await;

        // Check how many tasks are currently in progress
        let in_progress_count = orchestrator_tasks::Entity::find()
            .filter(orchestrator_tasks::Column::Status.eq("in_progress"))
            .count(db)
            .await
            .unwrap_or(0) as usize;

        let available_slots = parallel_limit.saturating_sub(in_progress_count);
        if available_slots == 0 {
            tracing::debug!("No available slots for new tasks (limit: {}, in_progress: {})", parallel_limit, in_progress_count);
            return;
        }

        // Fetch tasks from ClickUp
        let client = match ClickUpClient::from_env() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to create ClickUp client: {}", e);
                return;
            }
        };

        let tasks = match client.get_tasks(&list_id, Some(&trigger_status)).await {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("Failed to fetch tasks from ClickUp: {}", e);
                return;
            }
        };

        if tasks.is_empty() {
            tracing::debug!("No tasks found with status '{}'", trigger_status);
            return;
        }

        // Sort by priority (1=urgent first)
        let mut tasks = tasks;
        tasks.sort_by_key(|t| priority_to_int(&t.priority).unwrap_or(99));

        // Process up to available_slots tasks
        for task in tasks.into_iter().take(available_slots) {
            // Check if task already exists in database
            let existing = orchestrator_tasks::Entity::find()
                .filter(orchestrator_tasks::Column::ClickupTaskId.eq(&task.id))
                .one(db)
                .await;

            match existing {
                Ok(Some(_)) => {
                    tracing::debug!("Task {} already exists, skipping", task.id);
                    continue;
                }
                Err(e) => {
                    tracing::error!("Failed to check for existing task: {}", e);
                    continue;
                }
                Ok(None) => {}
            }

            tracing::info!("Processing new task: {} ({})", task.name, task.id);

            // Create worktree name from task name (sanitize)
            let worktree_name: String = task
                .name
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
                .collect::<String>()
                .to_lowercase();

            // Create a unique branch name for this task
            let task_branch = format!("task/{}-{}", task.id, worktree_name);
            let worktree_path = format!("{}/worktrees/{}", target_repo_path, worktree_name);

            // Update task status in ClickUp
            if let Err(e) = client.update_task_status(&task.id, &target_status).await {
                tracing::error!("Failed to update task status in ClickUp: {}", e);
                continue;
            }

            // Insert task into database
            let now = chrono::Utc::now();
            let new_task = orchestrator_tasks::ActiveModel {
                clickup_task_id: Set(task.id.clone()),
                clickup_list_id: Set(task.list.id.clone()),
                name: Set(task.name.clone()),
                description: Set(task.description.clone()),
                priority: Set(priority_to_int(&task.priority)),
                status: Set("in_progress".to_string()),
                worktree_path: Set(Some(worktree_path.clone())),
                time_spent_ms: Set(0),
                started_at: Set(Some(now.into())),
                completed_at: Set(None),
                created_at: Set(now.into()),
                updated_at: Set(now.into()),
                ..Default::default()
            };

            let inserted = match orchestrator_tasks::Entity::insert(new_task)
                .exec(db)
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Failed to insert task: {}", e);
                    continue;
                }
            };

            let task_id = inserted.last_insert_id;

            if let Err(e) = log_task_event(
                db,
                task_id,
                EVENT_SYSTEM,
                "Task created from ClickUp",
                None,
            )
            .await
            {
                tracing::warn!("Failed to log task creation for {}: {}", task_id, e);
            }

            if let Err(e) = log_task_event(
                db,
                task_id,
                EVENT_CLICKUP,
                format!("ClickUp status updated: {} -> {}", trigger_status, target_status),
                None,
            )
            .await
            {
                tracing::warn!(
                    "Failed to log ClickUp status update for {}: {}",
                    task_id,
                    e
                );
            }

            // Ensure worktrees directory exists
            let worktrees_dir = format!("{}/worktrees", target_repo_path);
            if let Err(e) = tokio::fs::create_dir_all(&worktrees_dir).await {
                tracing::error!("Failed to create worktrees directory: {}", e);
                let _ = orchestrator_tasks::Entity::update_many()
                    .filter(orchestrator_tasks::Column::Id.eq(task_id))
                    .col_expr(
                        orchestrator_tasks::Column::Status,
                        sea_orm::sea_query::Expr::value("failed"),
                    )
                    .exec(db)
                    .await;
                if let Err(log_err) = log_task_status_change(
                    db,
                    task_id,
                    "in_progress",
                    "failed",
                    Some(format!("worktrees dir create failed: {}", e)),
                )
                .await
                {
                    tracing::warn!(
                        "Failed to log worktree dir failure for {}: {}",
                        task_id,
                        log_err
                    );
                }
                continue;
            }

            // Fetch latest from remote before creating worktree
            let fetch_result = tokio::process::Command::new("git")
                .args(["-C", &target_repo_path, "fetch", "--all"])
                .output()
                .await;

            if let Err(e) = fetch_result {
                tracing::warn!("Failed to fetch from remote: {}", e);
                // Continue anyway, not fatal
            }

            // Create git worktree with a new branch based on dev_branch
            let worktree_result = tokio::process::Command::new("git")
                .args([
                    "-C",
                    &target_repo_path,
                    "worktree",
                    "add",
                    "-b",
                    &task_branch,
                    &worktree_path,
                    &dev_branch,
                ])
                .output()
                .await;

            match worktree_result {
                Err(e) => {
                    tracing::error!("Failed to run git worktree command: {}", e);
                    let _ = orchestrator_tasks::Entity::update_many()
                        .filter(orchestrator_tasks::Column::Id.eq(task_id))
                        .col_expr(
                            orchestrator_tasks::Column::Status,
                            sea_orm::sea_query::Expr::value("failed"),
                        )
                        .exec(db)
                        .await;
                    if let Err(log_err) = log_task_status_change(
                        db,
                        task_id,
                        "in_progress",
                        "failed",
                        Some(format!("git worktree command failed: {}", e)),
                    )
                    .await
                    {
                        tracing::warn!(
                            "Failed to log worktree command failure for {}: {}",
                            task_id,
                            log_err
                        );
                    }
                    continue;
                }
                Ok(output) if !output.status.success() => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::error!("Git worktree failed: {}", stderr);
                    let _ = orchestrator_tasks::Entity::update_many()
                        .filter(orchestrator_tasks::Column::Id.eq(task_id))
                        .col_expr(
                            orchestrator_tasks::Column::Status,
                            sea_orm::sea_query::Expr::value("failed"),
                        )
                        .exec(db)
                        .await;
                    if let Err(log_err) = log_task_status_change(
                        db,
                        task_id,
                        "in_progress",
                        "failed",
                        Some(format!("git worktree failed: {}", stderr)),
                    )
                    .await
                    {
                        tracing::warn!(
                            "Failed to log worktree failure for {}: {}",
                            task_id,
                            log_err
                        );
                    }
                    continue;
                }
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    tracing::info!("Created worktree at {} on branch {}: {}", worktree_path, task_branch, stdout.trim());
                    if let Err(log_err) = log_task_event(
                        db,
                        task_id,
                        EVENT_SYSTEM,
                        format!("Worktree created at {} (branch {})", worktree_path, task_branch),
                        None,
                    )
                    .await
                    {
                        tracing::warn!(
                            "Failed to log worktree creation for {}: {}",
                            task_id,
                            log_err
                        );
                    }
                }
            }

            // Verify the worktree directory exists before spawning
            if !std::path::Path::new(&worktree_path).exists() {
                tracing::error!("Worktree directory does not exist after creation: {}", worktree_path);
                let _ = orchestrator_tasks::Entity::update_many()
                    .filter(orchestrator_tasks::Column::Id.eq(task_id))
                    .col_expr(
                        orchestrator_tasks::Column::Status,
                        sea_orm::sea_query::Expr::value("failed"),
                    )
                    .exec(db)
                    .await;
                if let Err(log_err) = log_task_status_change(
                    db,
                    task_id,
                    "in_progress",
                    "failed",
                    Some("worktree directory missing after creation".to_string()),
                )
                .await
                {
                    tracing::warn!(
                        "Failed to log missing worktree dir for {}: {}",
                        task_id,
                        log_err
                    );
                }
                continue;
            }

            // Build prompt from task description combined with agent prompt
            let task_description = task
                .description
                .clone()
                .unwrap_or_else(|| format!("Complete task: {}", task.name));

            // Combine task description with global agent prompt if configured
            let prompt = match &agent_prompt {
                Some(global_prompt) if !global_prompt.is_empty() => {
                    format!(
                        "## Task\n{}\n\n## Instructions\n{}",
                        task_description, global_prompt
                    )
                }
                _ => task_description,
            };

            // Spawn CLI agent
            match PROCESS_MANAGER
                .spawn_agent(task_id, &prompt, &worktree_path)
                .await
            {
                Ok(pid) => {
                    tracing::info!(
                        "Spawned CLI agent for task {} (PID: {})",
                        task_id,
                        pid
                    );
                    if let Err(log_err) = log_task_event(
                        db,
                        task_id,
                        EVENT_SYSTEM,
                        format!("Agent spawned (PID: {})", pid),
                        None,
                    )
                    .await
                    {
                        tracing::warn!(
                            "Failed to log agent spawn for {}: {}",
                            task_id,
                            log_err
                        );
                    }

                    // Insert process session record
                    let session = crate::models::_entities::process_sessions::ActiveModel {
                        task_id: Set(task_id),
                        pid: Set(Some(pid as i32)),
                        started_at: Set(chrono::Utc::now().into()),
                        ended_at: Set(None),
                        exit_code: Set(None),
                        created_at: Set(chrono::Utc::now().into()),
                        updated_at: Set(chrono::Utc::now().into()),
                        ..Default::default()
                    };

                    let _ = crate::models::_entities::process_sessions::Entity::insert(session)
                        .exec(db)
                        .await;
                }
                Err(e) => {
                    tracing::error!("Failed to spawn CLI agent: {}", e);
                    // Update task status to failed
                    let _ = orchestrator_tasks::Entity::update_many()
                        .filter(orchestrator_tasks::Column::Id.eq(task_id))
                        .col_expr(
                            orchestrator_tasks::Column::Status,
                            sea_orm::sea_query::Expr::value("failed"),
                        )
                        .exec(db)
                        .await;
                    if let Err(log_err) = log_task_status_change(
                        db,
                        task_id,
                        "in_progress",
                        "failed",
                        Some(format!("agent spawn failed: {}", e)),
                    )
                    .await
                    {
                        tracing::warn!(
                            "Failed to log agent spawn failure for {}: {}",
                            task_id,
                            log_err
                        );
                    }
                }
            }
        }
    }
}

#[async_trait]
impl Initializer for ClickUpPollerInitializer {
    fn name(&self) -> String {
        "clickup-poller".to_string()
    }

    async fn after_routes(&self, router: Router, ctx: &AppContext) -> Result<Router> {
        // Spawn the polling task
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));

            loop {
                interval.tick().await;
                Self::poll_and_process(ctx_clone.clone()).await;
            }
        });

        tracing::info!("ClickUp poller started (polling every 30 seconds)");
        Ok(router)
    }
}
