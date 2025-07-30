use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    ApplicationFocus,
    ApplicationSwitch,
    WindowFocus,
    WindowSwitch,
    Idle,
    Active,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationActivity {
    pub id: String,
    pub activity_type: ActivityType,
    pub application_name: String,
    pub window_title: Option<String>,
    pub process_id: Option<u32>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: u32,
    pub focus_session_id: Option<String>, // 关联的专注会话ID
    pub is_productive: Option<bool>,      // 是否为生产性活动
}

impl Default for ApplicationActivity {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            activity_type: ActivityType::ApplicationFocus,
            application_name: String::new(),
            window_title: None,
            process_id: None,
            started_at: Utc::now(),
            ended_at: None,
            duration_seconds: 0,
            focus_session_id: None,
            is_productive: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySummary {
    pub date: DateTime<Utc>,
    pub total_active_time: u32,        // 总活跃时间（秒）
    pub total_idle_time: u32,          // 总空闲时间（秒）
    pub most_used_applications: Vec<ApplicationUsage>,
    pub productivity_score: Option<f32>, // 生产力得分
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationUsage {
    pub application_name: String,
    pub usage_time_seconds: u32,
    pub switch_count: u32,
    pub is_productive: Option<bool>,
} 