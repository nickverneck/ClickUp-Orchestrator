use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "settings",
            &[
                ("id", ColType::PkAuto),
                ("key", ColType::StringUniq),
                ("value", ColType::Text), // JSON serialized value
            ],
            &[],
        )
        .await?;

        // Seed default settings
        let db = m.get_connection();

        // Default settings
        let defaults = vec![
            ("parallel_limit", "1"),
            ("trigger_status", "Ready for Dev"),
            ("target_status", "In Development"),
            ("target_repo_path", ""),
            ("dev_branch", "dev"),
            ("clickup_workspace_id", ""),
            ("clickup_space_id", ""),
            ("clickup_folder_id", ""),
            ("clickup_list_id", ""),
        ];

        for (key, value) in defaults {
            db.execute_unprepared(&format!(
                "INSERT INTO settings (key, value, created_at, updated_at) VALUES ('{}', '{}', datetime('now'), datetime('now'))",
                key, value
            ))
            .await?;
        }

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "settings").await?;
        Ok(())
    }
}
