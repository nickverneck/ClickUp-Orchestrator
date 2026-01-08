use crate::models::_entities::orchestrator_task_logs;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};

pub const EVENT_OUTPUT: &str = "output";
pub const EVENT_STATUS: &str = "status";
pub const EVENT_CLICKUP: &str = "clickup";
pub const EVENT_SYSTEM: &str = "system";

pub async fn log_task_event(
    db: &DatabaseConnection,
    task_id: i32,
    event_type: &str,
    message: impl Into<String>,
    is_stderr: Option<bool>,
) -> Result<(), sea_orm::DbErr> {
    let now = chrono::Utc::now();
    let log = orchestrator_task_logs::ActiveModel {
        task_id: Set(task_id),
        event_type: Set(event_type.to_string()),
        message: Set(message.into()),
        is_stderr: Set(is_stderr),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };

    log.insert(db).await?;
    Ok(())
}

pub async fn log_task_status_change(
    db: &DatabaseConnection,
    task_id: i32,
    from_status: &str,
    to_status: &str,
    note: Option<String>,
) -> Result<(), sea_orm::DbErr> {
    let message = match note {
        Some(note) => format!("Status changed: {} -> {} ({})", from_status, to_status, note),
        None => format!("Status changed: {} -> {}", from_status, to_status),
    };

    log_task_event(db, task_id, EVENT_STATUS, message, None).await
}
