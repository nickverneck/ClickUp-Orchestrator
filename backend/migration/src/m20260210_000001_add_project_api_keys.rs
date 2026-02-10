use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Add clickup_api_key column to projects table
        m.alter_table(
            Table::alter()
                .table(Alias::new("projects"))
                .add_column(
                    ColumnDef::new(Alias::new("clickup_api_key"))
                        .text()
                        .null(),
                )
                .to_owned(),
        )
        .await?;

        // Migrate global API key from environment to default project
        let db = m.get_connection();

        // Try to read CLICKUP_API_KEY from environment
        let global_api_key = std::env::var("CLICKUP_API_KEY")
            .ok()
            .filter(|k| !k.is_empty());

        if let Some(api_key) = global_api_key {
            // Find the default project (created during multi-project migration)
            let default_project_result = db
                .query_one(
                    sea_orm::Statement::from_string(
                        sea_orm::DbBackend::Sqlite,
                        "SELECT id FROM projects WHERE name = 'Default Project' ORDER BY id LIMIT 1"
                            .to_string(),
                    ),
                )
                .await;

            if let Ok(Some(row)) = default_project_result {
                if let Ok(project_id) = row.try_get_by_index::<i32>(0) {
                    // Update the default project with the API key
                    let update_sql = format!(
                        "UPDATE projects SET clickup_api_key = '{}' WHERE id = {}",
                        api_key.replace("'", "''"),
                        project_id
                    );

                    db.execute_unprepared(&update_sql).await?;
                } else {
                    // Could not read project ID from row
                }
            } else {
                // No default project found - this is expected in fresh installations
            }
        }

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Drop clickup_api_key column from projects table
        m.alter_table(
            Table::alter()
                .table(Alias::new("projects"))
                .drop_column(Alias::new("clickup_api_key"))
                .to_owned(),
        )
        .await?;

        Ok(())
    }
}
