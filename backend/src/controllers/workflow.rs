//! Workflow controller for managing workflow configuration

use crate::models::_entities::workflow_configs;
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStatus {
    Running,
    Paused,
}

impl WorkflowStatus {
    fn from_db(value: &str) -> Self {
        match value {
            "running" => WorkflowStatus::Running,
            _ => WorkflowStatus::Paused,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            WorkflowStatus::Running => "running",
            WorkflowStatus::Paused => "paused",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowConfig {
    pub version: u32,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub label: String,
    pub position: WorkflowPosition,
    pub settings: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub output: Option<String>,
    pub actions: Vec<WorkflowAction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowAction {
    pub id: String,
    #[serde(rename = "type")]
    pub action_type: String,
    pub settings: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct WorkflowResponse {
    pub status: WorkflowStatus,
    pub config: WorkflowConfig,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub config: WorkflowConfig,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowStatusRequest {
    pub status: WorkflowStatus,
}

fn default_workflow_config() -> WorkflowConfig {
    WorkflowConfig {
        version: 1,
        nodes: vec![
            WorkflowNode {
                id: "node-queue".to_string(),
                node_type: "queue".to_string(),
                label: "Queue".to_string(),
                position: WorkflowPosition { x: 120.0, y: 180.0 },
                settings: json!({
                    "provider": "clickup",
                    "baseBranch": "dev",
                    "clickup": {
                        "listId": "",
                        "triggerStatus": "Ready for Dev"
                    }
                }),
            },
            WorkflowNode {
                id: "node-dev".to_string(),
                node_type: "agent".to_string(),
                label: "In Development".to_string(),
                position: WorkflowPosition { x: 420.0, y: 180.0 },
                settings: json!({
                    "systemPrompt": "",
                    "model": "codex",
                    "clickupStatus": "In Development",
                    "capacity": 2
                }),
            },
            WorkflowNode {
                id: "node-completed".to_string(),
                node_type: "bucket".to_string(),
                label: "Completed".to_string(),
                position: WorkflowPosition { x: 760.0, y: 80.0 },
                settings: json!({
                    "bucket": "completed"
                }),
            },
            WorkflowNode {
                id: "node-failed".to_string(),
                node_type: "bucket".to_string(),
                label: "Failed".to_string(),
                position: WorkflowPosition { x: 760.0, y: 260.0 },
                settings: json!({
                    "bucket": "failed"
                }),
            },
            WorkflowNode {
                id: "node-stopped".to_string(),
                node_type: "bucket".to_string(),
                label: "Stopped".to_string(),
                position: WorkflowPosition { x: 760.0, y: 440.0 },
                settings: json!({
                    "bucket": "stopped"
                }),
            },
        ],
        edges: vec![
            WorkflowEdge {
                id: "edge-queue-dev".to_string(),
                source: "node-queue".to_string(),
                target: "node-dev".to_string(),
                output: None,
                actions: vec![WorkflowAction {
                    id: "action-create-branch".to_string(),
                    action_type: "create_branch".to_string(),
                    settings: json!({
                        "branchPrefix": "task/"
                    }),
                }],
            },
            WorkflowEdge {
                id: "edge-dev-success".to_string(),
                source: "node-dev".to_string(),
                target: "node-completed".to_string(),
                output: Some("success".to_string()),
                actions: vec![WorkflowAction {
                    id: "action-complete-status".to_string(),
                    action_type: "update_clickup_status".to_string(),
                    settings: json!({
                        "status": "Complete"
                    }),
                }],
            },
            WorkflowEdge {
                id: "edge-dev-error".to_string(),
                source: "node-dev".to_string(),
                target: "node-failed".to_string(),
                output: Some("error".to_string()),
                actions: vec![WorkflowAction {
                    id: "action-failed-status".to_string(),
                    action_type: "update_clickup_status".to_string(),
                    settings: json!({
                        "status": "Failed"
                    }),
                }],
            },
        ],
    }
}

async fn fetch_or_create_workflow(
    db: &sea_orm::DatabaseConnection,
) -> Result<(workflow_configs::Model, WorkflowStatus, WorkflowConfig)> {
    if let Some(model) = workflow_configs::Entity::find()
        .order_by_asc(workflow_configs::Column::Id)
        .one(db)
        .await?
    {
        let status = WorkflowStatus::from_db(&model.status);
        let config = match serde_json::from_str::<WorkflowConfig>(&model.config) {
            Ok(config) => config,
            Err(_) => {
                let fallback = default_workflow_config();
                let mut active: workflow_configs::ActiveModel = model.clone().into();
                active.config = Set(serde_json::to_string(&fallback)?);
                active.updated_at = Set(chrono::Utc::now().into());
                active.update(db).await?;
                fallback
            }
        };
        return Ok((model, status, config));
    }

    let default_config = default_workflow_config();
    let now = chrono::Utc::now();
    let record = workflow_configs::ActiveModel {
        status: Set(WorkflowStatus::Paused.as_str().to_string()),
        config: Set(serde_json::to_string(&default_config)?),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };

    let inserted = record.insert(db).await?;
    Ok((inserted, WorkflowStatus::Paused, default_config))
}

#[debug_handler]
async fn get_workflow(State(ctx): State<AppContext>) -> Result<Response> {
    let (_, status, config) = fetch_or_create_workflow(&ctx.db).await?;
    format::json(WorkflowResponse { status, config })
}

#[debug_handler]
async fn update_workflow(
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateWorkflowRequest>,
) -> Result<Response> {
    let (model, status, _) = fetch_or_create_workflow(&ctx.db).await?;

    if status == WorkflowStatus::Running {
        return Err(Error::BadRequest(
            "Workflow is running. Pause before editing.".to_string(),
        ));
    }

    let mut active: workflow_configs::ActiveModel = model.into();
    active.config = Set(serde_json::to_string(&params.config)?);
    active.updated_at = Set(chrono::Utc::now().into());
    active.update(&ctx.db).await?;

    format::json(WorkflowResponse {
        status,
        config: params.config,
    })
}

async fn set_status(
    ctx: &AppContext,
    status: WorkflowStatus,
) -> Result<Response> {
    let (model, _, config) = fetch_or_create_workflow(&ctx.db).await?;
    let mut active: workflow_configs::ActiveModel = model.into();
    active.status = Set(status.as_str().to_string());
    active.updated_at = Set(chrono::Utc::now().into());
    active.update(&ctx.db).await?;

    format::json(WorkflowResponse { status, config })
}

#[debug_handler]
async fn update_status(
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateWorkflowStatusRequest>,
) -> Result<Response> {
    set_status(&ctx, params.status).await
}

#[debug_handler]
async fn start_workflow(State(ctx): State<AppContext>) -> Result<Response> {
    set_status(&ctx, WorkflowStatus::Running).await
}

#[debug_handler]
async fn pause_workflow(State(ctx): State<AppContext>) -> Result<Response> {
    set_status(&ctx, WorkflowStatus::Paused).await
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/workflow")
        .add("/", get(get_workflow))
        .add("/", put(update_workflow))
        .add("/status", put(update_status))
        .add("/start", post(start_workflow))
        .add("/pause", post(pause_workflow))
}
