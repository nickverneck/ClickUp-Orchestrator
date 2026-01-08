use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "orchestrator_task_logs",
            &[
                ("id", ColType::PkAuto),
                ("task_id", ColType::Integer),
                ("event_type", ColType::String),
                ("message", ColType::Text),
                ("is_stderr", ColType::BooleanNull),
            ],
            &[],
        )
        .await?;

        m.create_index(
            Index::create()
                .name("idx_task_logs_task_id")
                .table(Alias::new("orchestrator_task_logs"))
                .col(Alias::new("task_id"))
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "orchestrator_task_logs").await?;
        Ok(())
    }
}
