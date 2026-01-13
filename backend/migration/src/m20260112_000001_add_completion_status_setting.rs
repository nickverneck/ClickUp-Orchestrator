use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "INSERT OR IGNORE INTO settings (key, value, created_at, updated_at) VALUES ('completion_status', 'Complete', datetime('now'), datetime('now'))",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared("DELETE FROM settings WHERE key = 'completion_status'")
            .await?;
        Ok(())
    }
}
