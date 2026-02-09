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
                ("description", ColType::Text),
                ("status", ColType::String),        // 'active' or 'archived'
                ("repo_path", ColType::Text),        // Absolute filesystem path
                ("github_url", ColType::Text),       // GitHub clone URL (optional)
                ("dev_branch", ColType::String),     // Default 'dev'
                // ClickUp configuration
                ("clickup_workspace_id", ColType::Text),
                ("clickup_space_id", ColType::Text),
                ("clickup_folder_id", ColType::Text),
                ("clickup_list_id", ColType::Text),
                // Agent configuration
                ("agent_prompt", ColType::Text),
                ("agent_model", ColType::String),    // Default 'claude'
                ("parallel_limit", ColType::Integer),// Default 1
                ("created_at", ColType::Timestamp),
                ("updated_at", ColType::Timestamp),
            ],
            &[],
        )
        .await?;

        // Create index on status for quick lookup of active projects
        create_index(m, "projects", "idx_projects_status", &["status"], false).await?;

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

        create_index(
            m,
            "workflow_configs",
            "idx_workflow_configs_project_id",
            &["project_id"],
            false,
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

        create_index(
            m,
            "orchestrator_tasks",
            "idx_orchestrator_tasks_project_id",
            &["project_id"],
            false,
        )
        .await?;

        // Migrate existing settings to default project
        let db = m.get_connection();

        // Get current settings
        let settings_result = db
            .query_one(
                sea_orm::Statement::from_string(
                    sea_orm::DbBackend::Sqlite,
                    "SELECT target_repo_path, dev_branch, agent_prompt, agent_model, parallel_limit FROM settings LIMIT 1".to_string(),
                ),
            )
            .await;

        if let Ok(Some(row)) = settings_result {
            let repo_path: Option<String> = row.try_get_by_index(0).ok();
            let dev_branch: Option<String> = row.try_get_by_index(1).ok();
            let agent_prompt: Option<String> = row.try_get_by_index(2).ok();
            let agent_model: Option<String> = row.try_get_by_index(3).ok();
            let parallel_limit: Option<i32> = row.try_get_by_index(4).ok();

            let repo_path = repo_path.unwrap_or_else(|| "/tmp/clickup-orchestrator".to_string());
            let dev_branch = dev_branch.unwrap_or_else(|| "dev".to_string());
            let agent_model = agent_model.unwrap_or_else(|| "claude".to_string());
            let parallel_limit = parallel_limit.unwrap_or(1);

            let now = chrono::Utc::now().to_rfc3339();

            // Insert default project
            let insert_sql = format!(
                "INSERT INTO projects (name, description, status, repo_path, dev_branch, agent_prompt, agent_model, parallel_limit, created_at, updated_at) \
                 VALUES ('Default Project', 'Migrated from global settings', 'active', '{}', '{}', '{}', '{}', {}, '{}', '{}')",
                repo_path.replace("'", "''"),
                dev_branch.replace("'", "''"),
                agent_prompt.as_deref().unwrap_or("").replace("'", "''"),
                agent_model.replace("'", "''"),
                parallel_limit,
                now,
                now
            );

            db.execute_unprepared(&insert_sql).await?;

            // Get the inserted project ID and link existing workflows
            let project_id_result = db
                .query_one(
                    sea_orm::Statement::from_string(
                        sea_orm::DbBackend::Sqlite,
                        "SELECT id FROM projects WHERE name = 'Default Project' ORDER BY created_at DESC LIMIT 1".to_string(),
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
        }

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes
        drop_index(m, "projects", "idx_projects_status").await?;
        drop_index(m, "workflow_configs", "idx_workflow_configs_project_id").await?;
        drop_index(m, "orchestrator_tasks", "idx_orchestrator_tasks_project_id").await?;

        // Drop foreign key columns
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

        // Drop projects table
        drop_table(m, "projects").await?;

        Ok(())
    }
}
