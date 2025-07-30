use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub estimated_focus_sessions: u32, // 预计需要的专注时段
    pub completed_focus_sessions: u32, // 已完成的专注时段
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}

impl Default for Task {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: String::new(),
            description: None,
            priority: TaskPriority::Medium,
            status: TaskStatus::Pending,
            estimated_focus_sessions: 1,
            completed_focus_sessions: 0,
            created_at: now,
            updated_at: now,
            due_date: None,
            tags: Vec::new(),
        }
    }
} 