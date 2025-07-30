use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionType {
    Focus,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Pending,
    Active,
    Paused,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusSession {
    pub id: String,
    pub session_type: SessionType,
    pub status: SessionStatus,
    pub duration_minutes: u32,        // 计划时长（分钟）
    pub elapsed_seconds: u32,         // 已过去的时间（秒）
    pub task_id: Option<String>,      // 关联的任务ID
    pub started_at: Option<DateTime<Utc>>,
    pub paused_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub interruptions: u32,           // 中断次数
    pub notes: Option<String>,        // 会话笔记
}

impl Default for FocusSession {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            session_type: SessionType::Focus,
            status: SessionStatus::Pending,
            duration_minutes: 25,
            elapsed_seconds: 0,
            task_id: None,
            started_at: None,
            paused_at: None,
            completed_at: None,
            interruptions: 0,
            notes: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: u32,
    pub completed_sessions: u32,
    pub total_focus_time: u32,        // 总专注时间（分钟）
    pub average_session_length: f32,   // 平均会话时长（分钟）
    pub success_rate: f32,            // 成功率（百分比）
} 