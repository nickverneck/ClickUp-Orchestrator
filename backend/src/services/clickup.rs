//! ClickUp API client for hierarchy browsing and task operations

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const CLICKUP_API_BASE: &str = "https://api.clickup.com/api/v2";

#[derive(Error, Debug)]
pub enum ClickUpError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("API key not configured")]
    NoApiKey,
    #[error("ClickUp API error: {0}")]
    Api(String),
}

pub type Result<T> = std::result::Result<T, ClickUpError>;

/// ClickUp API client
pub struct ClickUpClient {
    client: Client,
    api_key: String,
}

// === API Response Types ===

#[derive(Debug, Deserialize, Serialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TeamsResponse {
    pub teams: Vec<Team>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Space {
    pub id: String,
    pub name: String,
    pub private: bool,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpacesResponse {
    pub spaces: Vec<Space>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub hidden: bool,
}

#[derive(Debug, Deserialize)]
pub struct FoldersResponse {
    pub folders: Vec<Folder>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct List {
    pub id: String,
    pub name: String,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListsResponse {
    pub lists: Vec<List>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ListDetails {
    pub id: String,
    pub name: String,
    pub statuses: Vec<Status>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Status {
    pub id: Option<String>,
    pub status: String,
    pub color: Option<String>,
    #[serde(rename = "type")]
    pub status_type: Option<String>,
    pub orderindex: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Option<TaskPriority>,
    pub list: TaskList,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskStatus {
    pub status: String,
    pub color: Option<String>,
    #[serde(rename = "type")]
    pub status_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskPriority {
    pub id: Option<String>,
    pub priority: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskList {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TasksResponse {
    pub tasks: Vec<Task>,
}

#[derive(Debug, Serialize)]
pub struct UpdateTaskRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TimeEntryRequest {
    pub start: i64,
    pub end: i64,
    pub time: i64,
}

impl ClickUpClient {
    /// Create a new ClickUp client
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Create a client from environment variable
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("CLICKUP_API_KEY").map_err(|_| ClickUpError::NoApiKey)?;
        Ok(Self::new(api_key))
    }

    /// Make an authenticated GET request
    async fn get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", CLICKUP_API_BASE, endpoint);
        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(ClickUpError::Api(format!("{}: {}", status, text)));
        }

        Ok(response.json().await?)
    }

    /// Make an authenticated PUT request
    async fn put<B: Serialize, T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", CLICKUP_API_BASE, endpoint);
        let response = self
            .client
            .put(&url)
            .header("Authorization", &self.api_key)
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(ClickUpError::Api(format!("{}: {}", status, text)));
        }

        Ok(response.json().await?)
    }

    /// Make an authenticated POST request
    async fn post<B: Serialize, T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", CLICKUP_API_BASE, endpoint);
        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.api_key)
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(ClickUpError::Api(format!("{}: {}", status, text)));
        }

        Ok(response.json().await?)
    }

    // === Hierarchy Browser Methods ===

    /// Get all workspaces (teams) the user has access to
    pub async fn get_workspaces(&self) -> Result<Vec<Team>> {
        let response: TeamsResponse = self.get("/team").await?;
        Ok(response.teams)
    }

    /// Get all spaces in a workspace
    pub async fn get_spaces(&self, team_id: &str) -> Result<Vec<Space>> {
        let response: SpacesResponse = self.get(&format!("/team/{}/space", team_id)).await?;
        Ok(response.spaces)
    }

    /// Get all folders in a space
    pub async fn get_folders(&self, space_id: &str) -> Result<Vec<Folder>> {
        let response: FoldersResponse = self.get(&format!("/space/{}/folder", space_id)).await?;
        Ok(response.folders)
    }

    /// Get all lists in a folder
    pub async fn get_lists_in_folder(&self, folder_id: &str) -> Result<Vec<List>> {
        let response: ListsResponse = self.get(&format!("/folder/{}/list", folder_id)).await?;
        Ok(response.lists)
    }

    /// Get folderless lists in a space
    pub async fn get_folderless_lists(&self, space_id: &str) -> Result<Vec<List>> {
        let response: ListsResponse = self.get(&format!("/space/{}/list", space_id)).await?;
        Ok(response.lists)
    }

    /// Get list details including statuses
    pub async fn get_list(&self, list_id: &str) -> Result<ListDetails> {
        self.get(&format!("/list/{}", list_id)).await
    }

    /// Get statuses for a list
    pub async fn get_list_statuses(&self, list_id: &str) -> Result<Vec<Status>> {
        let list: ListDetails = self.get_list(list_id).await?;
        Ok(list.statuses)
    }

    // === Task Operations ===

    /// Get tasks from a list with optional status filter
    pub async fn get_tasks(&self, list_id: &str, status: Option<&str>) -> Result<Vec<Task>> {
        let endpoint = match status {
            Some(s) => format!(
                "/list/{}/task?statuses[]={}",
                list_id,
                urlencoding::encode(s)
            ),
            None => format!("/list/{}/task", list_id),
        };
        let response: TasksResponse = self.get(&endpoint).await?;
        Ok(response.tasks)
    }

    /// Update a task's status
    pub async fn update_task_status(&self, task_id: &str, status: &str) -> Result<Task> {
        let body = UpdateTaskRequest {
            status: Some(status.to_string()),
        };
        self.put(&format!("/task/{}", task_id), &body).await
    }

    /// Add a time entry to a task
    pub async fn add_time_entry(
        &self,
        task_id: &str,
        start_ms: i64,
        end_ms: i64,
        duration_ms: i64,
    ) -> Result<serde_json::Value> {
        let body = TimeEntryRequest {
            start: start_ms,
            end: end_ms,
            time: duration_ms,
        };
        self.post(&format!("/task/{}/time", task_id), &body).await
    }
}

/// Helper to convert ClickUp priority to integer (1=urgent, 2=high, 3=normal, 4=low)
pub fn priority_to_int(priority: &Option<TaskPriority>) -> Option<i32> {
    priority.as_ref().and_then(|p| {
        p.priority.as_ref().and_then(|s| match s.as_str() {
            "urgent" => Some(1),
            "high" => Some(2),
            "normal" => Some(3),
            "low" => Some(4),
            _ => None,
        })
    })
}
