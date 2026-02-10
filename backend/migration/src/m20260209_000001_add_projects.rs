use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Create projects table
        create_table(
            m,
            "projects",
            &[
                ("id", ColType::PkAuto),
                ("name", ColType::String),
                ("description", ColType::TextNull),
                ("status", ColType::String),        // 'active' or 'archived'
                ("repo_path", ColType::Text),        // Absolute filesystem path
                ("github_url", ColType::TextNull),   // GitHub clone URL (optional)
                ("dev_branch", ColType::String),     // Default 'dev'
                // ClickUp configuration
                ("clickup_workspace_id", ColType::TextNull),
                ("clickup_space_id", ColType::TextNull),
                ("clickup_folder_id", ColType::TextNull),
                ("clickup_list_id", ColType::TextNull),
                // Agent configuration
                ("agent_prompt", ColType::TextNull),
                ("agent_model", ColType::String),    // Default 'claude'
                ("parallel_limit", ColType::Integer),// Default 1
            ],
            &[],
        )
        .await?;

        // Create index on status for quick lookup of active projects
        m.create_index(
            Index::create()
                .name("idx_projects_status")
                .table(Alias::new("projects"))
                .col(Alias::new("status"))
                .to_owned(),
        )
        .await?;

        // Add project_id to workflow_configs table
        m.alter_table(
            Table::alter()
                .table(Alias::new("workflow_configs"))
                .add_column(
                    ColumnDef::new(Alias::new("project_id"))
                        .integer()
                        .null(),
                )
                .to_owned(),
        )
        .await?;

        m.create_index(
            Index::create()
                .name("idx_workflow_configs_project_id")
                .table(Alias::new("workflow_configs"))
                .col(Alias::new("project_id"))
                .to_owned(),
        )
        .await?;

        // Add project_id to orchestrator_tasks table
        m.alter_table(
            Table::alter()
                .table(Alias::new("orchestrator_tasks"))
                .add_column(
                    ColumnDef::new(Alias::new("project_id"))
                        .integer()
                        .null(),
                )
                .to_owned(),
        )
        .await?;

        m.create_index(
            Index::create()
                .name("idx_orchestrator_tasks_project_id")
                .table(Alias::new("orchestrator_tasks"))
                .col(Alias::new("project_id"))
                .to_owned(),
        )
        .await?;

        // Migrate existing settings to default project
        let db = m.get_connection();

        // Get current settings - handle case where settings table might not have all columns
        let settings_result = db
            .query_all(
                sea_orm::Statement::from_string(
                    sea_orm::DbBackend::Sqlite,
                    "SELECT key, value FROM settings".to_string(),
                ),
            )
            .await;

        let mut settings_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        if let Ok(rows) = settings_result {
            for row in rows {
                if let (Ok(key), Ok(value)) = (row.try_get_by_index::<String>(0), row.try_get_by_index::<String>(1)) {
                    settings_map.insert(key, value);
                }
            }
        }

        let repo_path = settings_map
            .get("target_repo_path")
            .cloned()
            .unwrap_or_else(|| "/tmp/clickup-orchestrator".to_string());
        let dev_branch = settings_map
            .get("dev_branch")
            .cloned()
            .unwrap_or_else(|| "dev".to_string());
        let agent_model = settings_map
            .get("agent_model")
            .cloned()
            .unwrap_or_else(|| "claude".to_string());
        let agent_prompt = settings_map.get("agent_prompt").cloned();
        let parallel_limit = settings_map
            .get("parallel_limit")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1);

        // Insert default project using raw SQL with proper datetime handling
        let insert_sql = format!(
            "INSERT INTO projects (name, description, status, repo_path, dev_branch, agent_prompt, agent_model, parallel_limit) \
             VALUES ('Default Project', 'Migrated from global settings', 'active', '{}', '{}', '{}', '{}', {})",
            repo_path.replace("'", "''"),
            dev_branch.replace("'", "''"),
            agent_prompt.as_deref().unwrap_or("").replace("'", "''"),
            agent_model.replace("'", "''"),
            parallel_limit
        );

        db.execute_unprepared(&insert_sql).await?;

        // Get the inserted project ID and link existing workflows
        let project_id_result = db
            .query_one(
                sea_orm::Statement::from_string(
                    sea_orm::DbBackend::Sqlite,
                    "SELECT id FROM projects WHERE name = 'Default Project' ORDER BY id DESC LIMIT 1".to_string(),
                ),
            )
            .await;

        if let Ok(Some(row)) = project_id_result {
            if let Ok(project_id) = row.try_get_by_index::<i32>(0) {
                // Link all existing workflows to default project
                db.execute_unprepared(&format!(
                    "UPDATE workflow_configs SET project_id = {} WHERE project_id IS NULL",
                    project_id
                ))
                .await?;

                // Link all existing tasks to default project
                db.execute_unprepared(&format!(
                    "UPDATE orchestrator_tasks SET project_id = {} WHERE project_id IS NULL",
                    project_id
                ))
                .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Drop foreign key columns first
        m.alter_table(
            Table::alter()
                .table(Alias::new("workflow_configs"))
                .drop_column(Alias::new("project_id"))
                .to_owned(),
        )
        .await?;

        m.alter_table(
            Table::alter()
                .table(Alias::new("orchestrator_tasks"))
                .drop_column(Alias::new("project_id"))
                .to_owned(),
        )
        .await?;

        // Drop projects table (which also drops its indexes)
        drop_table(m, "projects").await?;

        Ok(())
    }
}
