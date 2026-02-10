//! Setup controller for first-time configuration

use crate::models::_entities::projects;
use loco_rs::prelude::*;
use sea_orm::EntityTrait;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SetupStatus {
    pub is_complete: bool,
    pub has_project: bool,
}

/// Get setup status
#[debug_handler]
async fn get_status(State(ctx): State<AppContext>) -> Result<Response> {
    // Check if at least one project exists
    let has_project = projects::Entity::find()
        .one(&ctx.db)
        .await
        .ok()
        .flatten()
        .is_some();

    // Setup is complete if there's at least one project
    let is_complete = has_project;

    format::json(SetupStatus {
        is_complete,
        has_project,
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/setup")
        .add("/status", get(get_status))
}
