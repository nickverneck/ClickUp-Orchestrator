use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Add opencode_model column to projects table
        m.alter_table(
            Table::alter()
                .table(Alias::new("projects"))
                .add_column(
                    ColumnDef::new(Alias::new("opencode_model"))
                        .text()
                        .null(),
                )
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("projects"))
                .drop_column(Alias::new("opencode_model"))
                .to_owned(),
        )
        .await?;

        Ok(())
    }
}
