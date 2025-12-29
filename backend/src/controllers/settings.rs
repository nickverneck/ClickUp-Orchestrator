//! Settings controller for managing application configuration

use crate::models::_entities::settings;
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct SettingsResponse {
    pub settings: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub settings: HashMap<String, String>,
}

/// Get all settings as key-value pairs
#[debug_handler]
async fn get_all(State(ctx): State<AppContext>) -> Result<Response> {
    let all_settings = settings::Entity::find().all(&ctx.db).await?;

    let settings_map: HashMap<String, String> = all_settings
        .into_iter()
        .map(|s| (s.key, s.value))
        .collect();

    format::json(SettingsResponse {
        settings: settings_map,
    })
}

/// Get a single setting by key
#[debug_handler]
async fn get_one(State(ctx): State<AppContext>, Path(key): Path<String>) -> Result<Response> {
    let setting = settings::Entity::find()
        .filter(settings::Column::Key.eq(&key))
        .one(&ctx.db)
        .await?;

    match setting {
        Some(s) => format::json(serde_json::json!({ "key": s.key, "value": s.value })),
        None => Err(Error::NotFound),
    }
}

/// Update multiple settings at once
#[debug_handler]
async fn update_all(
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateSettingsRequest>,
) -> Result<Response> {
    for (key, value) in params.settings {
        let existing = settings::Entity::find()
            .filter(settings::Column::Key.eq(&key))
            .one(&ctx.db)
            .await?;

        match existing {
            Some(setting) => {
                let mut active: settings::ActiveModel = setting.into();
                active.value = sea_orm::ActiveValue::Set(value);
                active.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
                active.update(&ctx.db).await?;
            }
            None => {
                let new_setting = settings::ActiveModel {
                    key: sea_orm::ActiveValue::Set(key),
                    value: sea_orm::ActiveValue::Set(value),
                    created_at: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
                    updated_at: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
                    ..Default::default()
                };
                new_setting.insert(&ctx.db).await?;
            }
        }
    }

    // Return updated settings
    get_all(State(ctx)).await
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/settings")
        .add("/", get(get_all))
        .add("/", put(update_all))
        .add("/{key}", get(get_one))
}
