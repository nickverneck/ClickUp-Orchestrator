use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "process_sessions",
            &[
                ("id", ColType::PkAuto),
                ("task_id", ColType::Integer),
                ("pid", ColType::IntegerNull),
                ("started_at", ColType::TimestampWithTimeZone),
                ("ended_at", ColType::TimestampWithTimeZoneNull),
                ("exit_code", ColType::IntegerNull),
            ],
            &[],
        )
        .await?;

        // Add index on task_id for faster lookups
        m.create_index(
            Index::create()
                .name("idx_process_sessions_task_id")
                .table(Alias::new("process_sessions"))
                .col(Alias::new("task_id"))
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "process_sessions").await?;
        Ok(())
    }
}
