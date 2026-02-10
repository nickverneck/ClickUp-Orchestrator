//! ClickUp Poller Initializer
//!
//! Starts a background task that polls ClickUp for new tasks and processes them.
//! Now supports multiple projects with per-project ClickUp configurations.

use async_trait::async_trait;
use axum::Router;
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    Set,
};
use std::time::Duration;
use tokio::time::interval;

use crate::models::_entities::{orchestrator_tasks, projects, settings};
use crate::services::clickup::{priority_to_int, ClickUpClient};
use crate::services::process_manager::PROCESS_MANAGER;
use crate::services::task_logs::{
    log_task_event, log_task_status_change, EVENT_CLICKUP, EVENT_SYSTEM,
};

pub struct ClickUpPollerInitializer;

struct PendingTask {
    id: Option<i32>,
    clickup_task_id: String,
    clickup_list_id: String,
    name: String,
    description: Option<String>,
    priority: Option<i32>,
    previous_status: Option<String>,
}

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

    fn sanitize_worktree_name(name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
            .collect::<String>()
            .to_lowercase()
    }

    async fn start_task(
        db: &sea_orm::DatabaseConnection,
        client: &ClickUpClient,
        pending: PendingTask,
        trigger_status: &str,
        target_status: &str,
        target_repo_path: &str,
        dev_branch: &str,
        agent_prompt: &Option<String>,
        agent_model: &str,
        project_id: Option<i32>,
    ) -> Result<i32, ()> {
        tracing::info!(
            "Processing task: {} ({})",
            pending.name,
            pending.clickup_task_id
        );

        let worktree_name = Self::sanitize_worktree_name(&pending.name);
        let task_branch = format!("task/{}-{}", pending.clickup_task_id, worktree_name);
        let worktree_path = format!("{}/worktrees/{}", target_repo_path, worktree_name);

        if let Err(e) = client
            .update_task_status(&pending.clickup_task_id, target_status)
            .await
        {
            tracing::error!("Failed to update task status in ClickUp: {}", e);
            if let Some(task_id) = pending.id {
                let _ = log_task_event(
                    db,
                    task_id,
                    EVENT_CLICKUP,
                    format!("ClickUp status update failed: {}", e),
                    None,
                )
                .await;
            }
            return Err(());
        }

        let now = chrono::Utc::now();
        let task_id = match pending.id {
            Some(existing_id) => {
                let mut active: orchestrator_tasks::ActiveModel = match orchestrator_tasks::Entity::find_by_id(existing_id)
                    .one(db)
                    .await
                {
                    Ok(Some(task)) => task.into(),
                    Ok(None) => {
                        tracing::error!("Queued task {} not found", existing_id);
                        return Err(());
                    }
                    Err(e) => {
                        tracing::error!("Failed to load queued task {}: {}", existing_id, e);
                        return Err(());
                    }
                };

                active.status = Set("in_progress".to_string());
                active.worktree_path = Set(Some(worktree_path.clone()));
                active.started_at = Set(Some(now.into()));
                active.completed_at = Set(None);
                active.updated_at = Set(now.into());

                if let Err(e) = active.update(db).await {
                    tracing::error!("Failed to update queued task {}: {}", existing_id, e);
                    return Err(());
                }

                if let Some(previous_status) = pending.previous_status.as_deref() {
                    if let Err(e) = log_task_status_change(
                        db,
                        existing_id,
                        previous_status,
                        "in_progress",
                        Some("slot available".to_string()),
                    )
                    .await
                    {
                        tracing::warn!(
                            "Failed to log queue start for task {}: {}",
                            existing_id,
                            e
                        );
                    }
                }

                existing_id
            }
            None => {
                let new_task = orchestrator_tasks::ActiveModel {
                    clickup_task_id: Set(pending.clickup_task_id.clone()),
                    clickup_list_id: Set(pending.clickup_list_id.clone()),
                    name: Set(pending.name.clone()),
                    description: Set(pending.description.clone()),
                    priority: Set(pending.priority),
                    status: Set("in_progress".to_string()),
                    worktree_path: Set(Some(worktree_path.clone())),
                    time_spent_ms: Set(0),
                    started_at: Set(Some(now.into())),
                    completed_at: Set(None),
                    project_id: Set(project_id),
                    created_at: Set(now.into()),
                    updated_at: Set(now.into()),
                    ..Default::default()
                };

                let inserted = match orchestrator_tasks::Entity::insert(new_task).exec(db).await
                {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("Failed to insert task: {}", e);
                        return Err(());
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

                task_id
            }
        };

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
            return Err(());
        }

        let fetch_result = tokio::process::Command::new("git")
            .args(["-C", target_repo_path, "fetch", "--all"])
            .output()
            .await;

        if let Err(e) = fetch_result {
            tracing::warn!("Failed to fetch from remote: {}", e);
        }

        let worktree_result = tokio::process::Command::new("git")
            .args([
                "-C",
                target_repo_path,
                "worktree",
                "add",
                "-b",
                &task_branch,
                &worktree_path,
                dev_branch,
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
                return Err(());
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
                return Err(());
            }
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                tracing::info!(
                    "Created worktree at {} on branch {}: {}",
                    worktree_path,
                    task_branch,
                    stdout.trim()
                );
                if let Err(log_err) = log_task_event(
                    db,
                    task_id,
                    EVENT_SYSTEM,
                    format!(
                        "Worktree created at {} (branch {})",
                        worktree_path, task_branch
                    ),
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

        if !std::path::Path::new(&worktree_path).exists() {
            tracing::error!(
                "Worktree directory does not exist after creation: {}",
                worktree_path
            );
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
            return Err(());
        }

        let task_description = pending
            .description
            .clone()
            .unwrap_or_else(|| format!("Complete task: {}", pending.name));

        let prompt = match agent_prompt {
            Some(global_prompt) if !global_prompt.is_empty() => {
                format!("## Task\n{}\n\n## Instructions\n{}", task_description, global_prompt)
            }
            _ => task_description,
        };

        match PROCESS_MANAGER
            .spawn_agent(task_id, &prompt, &worktree_path, agent_model)
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
                    format!("Agent spawned (model: {}, PID: {})", agent_model, pid),
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
                return Err(());
            }
        }

        Ok(task_id)
    }

    async fn poll_and_process(ctx: AppContext) {
        let db = &ctx.db;

        // First, try new multi-project approach
        let projects_result = projects::Entity::find()
            .filter(projects::Column::Status.eq("active"))
            .all(db)
            .await;

        match projects_result {
            Ok(active_projects) if !active_projects.is_empty() => {
                // Process each active project
                for project in active_projects {
                    Self::poll_and_process_project(db, &project).await;
                }
            }
            _ => {
                // Fall back to legacy single-project mode
                Self::poll_and_process_legacy(db).await;
            }
        }
    }

    async fn poll_and_process_project(
        db: &sea_orm::DatabaseConnection,
        project: &projects::Model,
    ) {
        // Check if project has ClickUp API key configured
        let api_key = match project.clickup_api_key.as_deref() {
            Some(key) if !key.is_empty() => key.to_string(),
            _ => {
                tracing::warn!(
                    "Project {} has no ClickUp API key configured, skipping",
                    project.name
                );
                return;
            }
        };

        let list_id = match project.clickup_list_id.as_deref() {
            Some(id) if !id.is_empty() => id.to_string(),
            _ => {
                tracing::debug!(
                    "Project {} has no ClickUp list configured, skipping",
                    project.name
                );
                return;
            }
        };

        let trigger_status = "Ready for Dev".to_string();  // Could be configurable per project
        let target_status = "In Development".to_string();  // Could be configurable per project
        let parallel_limit = project.parallel_limit as usize;
        let target_repo_path = project.repo_path.clone();
        let dev_branch = project.dev_branch.clone();
        let agent_prompt = project.agent_prompt.clone();
        let agent_model = project.agent_model.clone();
        let project_id = project.id;

        Self::poll_and_process_list(
            db,
            &list_id,
            &trigger_status,
            &target_status,
            parallel_limit,
            &target_repo_path,
            &dev_branch,
            &agent_prompt,
            &agent_model,
            Some(project_id),
            &api_key,
        )
        .await;
    }

    async fn poll_and_process_legacy(db: &sea_orm::DatabaseConnection) {
        // Get global API key from environment (legacy single-project mode)
        let api_key = match std::env::var("CLICKUP_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                tracing::debug!("No global CLICKUP_API_KEY configured (legacy), skipping poll");
                return;
            }
        };

        // Get global settings (legacy single-project mode)
        let Some(list_id) = Self::get_setting(db, "clickup_list_id").await else {
            tracing::debug!("No ClickUp list configured (legacy), skipping poll");
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
                tracing::debug!("No target repo path configured (legacy), skipping poll");
                return;
            }
        };

        let dev_branch = Self::get_setting(db, "dev_branch")
            .await
            .unwrap_or_else(|| "dev".to_string());

        let agent_prompt = Self::get_setting(db, "agent_prompt").await;
        let agent_model = Self::get_setting(db, "agent_model")
            .await
            .unwrap_or_else(|| "claude".to_string());

        Self::poll_and_process_list(
            db,
            &list_id,
            &trigger_status,
            &target_status,
            parallel_limit,
            &target_repo_path,
            &dev_branch,
            &agent_prompt,
            &agent_model,
            None,
            &api_key,
        )
        .await;
    }

    async fn poll_and_process_list(
        db: &sea_orm::DatabaseConnection,
        list_id: &str,
        trigger_status: &str,
        target_status: &str,
        parallel_limit: usize,
        target_repo_path: &str,
        dev_branch: &str,
        agent_prompt: &Option<String>,
        agent_model: &str,
        project_id: Option<i32>,
        api_key: &str,
    ) {

        // Check how many tasks are currently in progress (for this project)
        let mut in_progress_query = orchestrator_tasks::Entity::find()
            .filter(orchestrator_tasks::Column::Status.eq("in_progress"));

        if let Some(pid) = project_id {
            in_progress_query = in_progress_query.filter(orchestrator_tasks::Column::ProjectId.eq(pid));
        }

        let in_progress_count = in_progress_query
            .count(db)
            .await
            .unwrap_or(0) as usize;

        let mut available_slots = parallel_limit.saturating_sub(in_progress_count);
        if available_slots == 0 {
            tracing::debug!(
                "No available slots for new tasks (limit: {}, in_progress: {})",
                parallel_limit,
                in_progress_count
            );
        }

        // Fetch tasks from ClickUp
        let client = ClickUpClient::new(api_key.to_string());

        if available_slots > 0 {
            let mut queued_query = orchestrator_tasks::Entity::find()
                .filter(orchestrator_tasks::Column::Status.eq("queued"))
                .order_by_asc(orchestrator_tasks::Column::Priority)
                .order_by_asc(orchestrator_tasks::Column::CreatedAt)
                .limit(available_slots as u64);

            if let Some(pid) = project_id {
                queued_query = queued_query.filter(orchestrator_tasks::Column::ProjectId.eq(pid));
            }

            let queued_tasks = match queued_query.all(db).await {
                Ok(tasks) => tasks,
                Err(e) => {
                    tracing::error!("Failed to load queued tasks: {}", e);
                    Vec::new()
                }
            };

            for queued in queued_tasks {
                let pending = PendingTask {
                    id: Some(queued.id),
                    clickup_task_id: queued.clickup_task_id.clone(),
                    clickup_list_id: queued.clickup_list_id.clone(),
                    name: queued.name.clone(),
                    description: queued.description.clone(),
                    priority: queued.priority,
                    previous_status: Some(queued.status.clone()),
                };

                if Self::start_task(
                    db,
                    &client,
                    pending,
                    trigger_status,
                    target_status,
                    target_repo_path,
                    dev_branch,
                    agent_prompt,
                    agent_model,
                    project_id,
                )
                .await
                .is_ok()
                {
                    available_slots = available_slots.saturating_sub(1);
                }

                if available_slots == 0 {
                    break;
                }
            }
        }

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

        // Process tasks (start if slot available, otherwise queue)
        for task in tasks.into_iter() {
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

            if available_slots > 0 {
                let pending = PendingTask {
                    id: None,
                    clickup_task_id: task.id.clone(),
                    clickup_list_id: task.list.id.clone(),
                    name: task.name.clone(),
                    description: task.description.clone(),
                    priority: priority_to_int(&task.priority),
                    previous_status: None,
                };

                if Self::start_task(
                    db,
                    &client,
                    pending,
                    trigger_status,
                    target_status,
                    target_repo_path,
                    dev_branch,
                    agent_prompt,
                    agent_model,
                    project_id,
                )
                .await
                .is_ok()
                {
                    available_slots = available_slots.saturating_sub(1);
                }
            } else {
                let now = chrono::Utc::now();
                let new_task = orchestrator_tasks::ActiveModel {
                    clickup_task_id: Set(task.id.clone()),
                    clickup_list_id: Set(task.list.id.clone()),
                    name: Set(task.name.clone()),
                    description: Set(task.description.clone()),
                    priority: Set(priority_to_int(&task.priority)),
                    status: Set("queued".to_string()),
                    worktree_path: Set(None),
                    time_spent_ms: Set(0),
                    started_at: Set(None),
                    completed_at: Set(None),
                    project_id: Set(project_id),
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
                        tracing::error!("Failed to insert queued task: {}", e);
                        continue;
                    }
                };

                if let Err(e) = log_task_event(
                    db,
                    inserted.last_insert_id,
                    EVENT_SYSTEM,
                    "Task queued from ClickUp (waiting for slot)",
                    None,
                )
                .await
                {
                    tracing::warn!(
                        "Failed to log queued task creation for {}: {}",
                        inserted.last_insert_id,
                        e
                    );
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
