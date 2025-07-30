use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub user_id: String,
    pub username: String,
    pub default_focus_duration: u32, // 默认专注时间（分钟）
    pub short_break_duration: u32,   // 短休息时间（分钟）
    pub long_break_duration: u32,    // 长休息时间（分钟）
    pub notification_enabled: bool,   // 是否启用通知
    pub sound_enabled: bool,         // 是否启用声音
    pub theme: String,               // 主题设置
    pub auto_start_break: bool,      // 是否自动开始休息
    pub auto_start_focus: bool,      // 是否自动开始专注
    
    // 监控相关设置
    pub whitelist: Vec<String>,      // 白名单应用
    pub blacklist: Vec<String>,      // 黑名单应用
    pub autostart: bool,             // 是否自动启动
    pub fatigue_notify: bool,        // 是否启用疲劳提醒
    
    // 分心干预设置
    pub distraction_intervention: DistractionInterventionSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistractionInterventionSettings {
    pub enabled: bool,                    // 是否启用分心干预
    pub light_distraction_notification: bool,  // 轻度分心通知
    pub severe_distraction_popup: bool,   // 严重分心弹窗
    pub encouragement_enabled: bool,      // 是否启用鼓励消息
    pub intervention_cooldown_minutes: u32, // 干预冷却时间（分钟）
    pub notification_sound: bool,         // 干预通知是否播放声音
    pub popup_duration_seconds: u32,     // 弹窗显示时长（秒）
    pub encouragement_frequency: String, // 鼓励频率 ("low", "medium", "high")
}

impl Default for DistractionInterventionSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            light_distraction_notification: true,
            severe_distraction_popup: true,
            encouragement_enabled: true,
            intervention_cooldown_minutes: 5,
            notification_sound: true,
            popup_duration_seconds: 10,
            encouragement_frequency: "medium".to_string(),
        }
    }
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            user_id: String::new(),
            username: String::new(),
            default_focus_duration: 25,
            short_break_duration: 5,
            long_break_duration: 15,
            notification_enabled: true,
            sound_enabled: true,
            theme: "light".to_string(),
            auto_start_break: false,
            auto_start_focus: false,
            whitelist: Vec::new(),
            blacklist: Vec::new(),
            autostart: false,
            fatigue_notify: true,
            distraction_intervention: DistractionInterventionSettings::default(),
        }
    }
} 