use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("workflow_configs"))
                .add_column(
                    ColumnDef::new(Alias::new("name"))
                        .string()
                        .not_null()
                        .default("Default Workflow"),
                )
                .to_owned(),
        )
        .await?;

        let db = m.get_connection();
        db.execute_unprepared(
            "UPDATE workflow_configs SET name = 'Default Workflow' WHERE name IS NULL OR name = ''",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("workflow_configs"))
                .drop_column(Alias::new("name"))
                .to_owned(),
        )
        .await?;
        Ok(())
    }
}
