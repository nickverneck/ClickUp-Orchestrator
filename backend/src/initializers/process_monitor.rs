//! Process Monitor Initializer
//!
//! Listens for process exit events and updates the database accordingly.
//! This ensures that when an agent finishes its work, the task is properly
//! marked as completed/failed and the next queued task can be processed.

use async_trait::async_trait;
use axum::Router;
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::models::_entities::{orchestrator_tasks, process_sessions};
use crate::services::process_manager::{ProcessExitEvent, PROCESS_MANAGER};
use crate::services::task_logs::log_task_status_change;

pub struct ProcessMonitorInitializer;

impl ProcessMonitorInitializer {
    async fn handle_process_exit(db: &sea_orm::DatabaseConnection, event: ProcessExitEvent) {
        let now = chrono::Utc::now();

        // Determine final status based on exit code
        let final_status = if event.exit_code == 0 {
            "completed"
        } else {
            "failed"
        };

        // Update orchestrator_tasks
        let task_result = orchestrator_tasks::Entity::find_by_id(event.task_id)
            .one(db)
            .await;

        if let Ok(Some(task)) = task_result {
            let previous_status = task.status.clone();
            let time_spent_ms = if task.status == "in_progress" {
                match task.started_at.as_ref() {
                    Some(started_at) => {
                        let elapsed_ms = now
                            .signed_duration_since(started_at.with_timezone(&chrono::Utc))
                            .num_milliseconds();
                        let elapsed_ms = if elapsed_ms <= 0 {
                            0
                        } else if elapsed_ms > i32::MAX as i64 {
                            i32::MAX
                        } else {
                            elapsed_ms as i32
                        };
                        task.time_spent_ms.saturating_add(elapsed_ms)
                    }
                    None => task.time_spent_ms,
                }
            } else {
                task.time_spent_ms
            };

            let mut active: orchestrator_tasks::ActiveModel = task.into();
            active.status = Set(final_status.to_string());
            active.completed_at = Set(Some(now.into()));
            active.time_spent_ms = Set(time_spent_ms);
            active.output_log = Set(Some(event.output_log));
            active.updated_at = Set(now.into());

            if let Err(e) = active.update(db).await {
                tracing::error!("Failed to update task {} on exit: {}", event.task_id, e);
            } else {
                tracing::info!(
                    "Task {} marked as {} (exit code: {}, time: {}ms)",
                    event.task_id,
                    final_status,
                    event.exit_code,
                    time_spent_ms
                );
                if let Err(e) = log_task_status_change(
                    db,
                    event.task_id,
                    &previous_status,
                    final_status,
                    Some(format!("exit code {}", event.exit_code)),
                )
                .await
                {
                    tracing::warn!(
                        "Failed to log status change for task {}: {}",
                        event.task_id,
                        e
                    );
                }
            }
        } else {
            tracing::warn!(
                "Could not find task {} to update on exit",
                event.task_id
            );
        }

        // Update process_sessions
        let session_update = process_sessions::Entity::update_many()
            .filter(process_sessions::Column::TaskId.eq(event.task_id))
            .filter(process_sessions::Column::EndedAt.is_null())
            .col_expr(
                process_sessions::Column::EndedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .col_expr(
                process_sessions::Column::ExitCode,
                sea_orm::sea_query::Expr::value(event.exit_code),
            )
            .col_expr(
                process_sessions::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .exec(db)
            .await;

        if let Err(e) = session_update {
            tracing::error!(
                "Failed to update process session for task {}: {}",
                event.task_id,
                e
            );
        }
    }
}

#[async_trait]
impl Initializer for ProcessMonitorInitializer {
    fn name(&self) -> String {
        "process-monitor".to_string()
    }

    async fn after_routes(&self, router: Router, ctx: &AppContext) -> Result<Router> {
        let ctx_clone = ctx.clone();

        tokio::spawn(async move {
            let mut exit_rx = PROCESS_MANAGER.subscribe_exits();

            loop {
                match exit_rx.recv().await {
                    Ok(event) => {
                        tracing::info!(
                            "Process exit event received: task_id={}, exit_code={}",
                            event.task_id,
                            event.exit_code
                        );
                        Self::handle_process_exit(&ctx_clone.db, event).await;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("Process monitor lagged by {} events", n);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::error!("Process exit channel closed unexpectedly");
                        break;
                    }
                }
            }
        });

        tracing::info!("Process monitor started - listening for process exit events");
        Ok(router)
    }
}
