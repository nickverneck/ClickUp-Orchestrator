//! Task log listener initializer
//!
//! Persists process output lines to the task logs table for historical inspection.

use async_trait::async_trait;
use axum::Router;
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};
use tokio::sync::broadcast;

use crate::services::process_manager::PROCESS_MANAGER;
use crate::services::task_logs::{log_task_event, EVENT_OUTPUT};

pub struct TaskLogListenerInitializer;

#[async_trait]
impl Initializer for TaskLogListenerInitializer {
    fn name(&self) -> String {
        "task-log-listener".to_string()
    }

    async fn after_routes(&self, router: Router, ctx: &AppContext) -> Result<Router> {
        let ctx_clone = ctx.clone();

        tokio::spawn(async move {
            let mut output_rx = PROCESS_MANAGER.subscribe_output();

            loop {
                match output_rx.recv().await {
                    Ok(output) => {
                        if let Err(e) = log_task_event(
                            &ctx_clone.db,
                            output.task_id,
                            EVENT_OUTPUT,
                            output.line,
                            Some(output.is_stderr),
                        )
                        .await
                        {
                            tracing::warn!(
                                "Failed to persist output log for task {}: {}",
                                output.task_id,
                                e
                            );
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        });

        tracing::info!("Task log listener started - persisting task output logs");
        Ok(router)
    }
}
