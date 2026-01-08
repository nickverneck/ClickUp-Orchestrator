#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20251228_205515_orchestrator_tasks;
mod m20251228_205522_process_sessions;
mod m20251228_205527_settings;
mod m20260107_000001_add_task_output_log;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20251228_205515_orchestrator_tasks::Migration),
            Box::new(m20251228_205522_process_sessions::Migration),
            Box::new(m20251228_205527_settings::Migration),
            Box::new(m20260107_000001_add_task_output_log::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}