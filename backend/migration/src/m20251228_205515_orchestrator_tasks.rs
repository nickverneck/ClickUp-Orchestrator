use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "orchestrator_tasks",
            &[
                ("id", ColType::PkAuto),
                ("clickup_task_id", ColType::StringUniq),
                ("clickup_list_id", ColType::String),
                ("name", ColType::String),
                ("description", ColType::TextNull),
                ("priority", ColType::IntegerNull), // 1=urgent, 2=high, 3=normal, 4=low
                ("status", ColType::String),        // queued, in_progress, stopped, completed, failed
                ("worktree_path", ColType::StringNull),
                ("time_spent_ms", ColType::Integer), // milliseconds
                ("started_at", ColType::TimestampWithTimeZoneNull),
                ("completed_at", ColType::TimestampWithTimeZoneNull),
            ],
            &[],
        )
        .await?;

        // Add index on status for faster filtering
        m.create_index(
            Index::create()
                .name("idx_orchestrator_tasks_status")
                .table(Alias::new("orchestrator_tasks"))
                .col(Alias::new("status"))
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "orchestrator_tasks").await?;
        Ok(())
    }
}
