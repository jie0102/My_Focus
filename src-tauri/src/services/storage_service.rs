use std::path::PathBuf;
use std::fs;
use anyhow::Result;
use serde_json;
use crate::commands::{UserSettings, Task};
use crate::models::{FocusSession, ApplicationActivity};
use crate::services::ai_service::AIConfig;

pub struct StorageService {
    data_dir: PathBuf,
}

impl StorageService {
    pub fn new(data_dir: PathBuf) -> Self {
        // 确保数据目录存在
        if !data_dir.exists() {
            let _ = fs::create_dir_all(&data_dir);
        }
        Self { data_dir }
    }

    pub async fn save_user_settings(&self, settings: &UserSettings) -> Result<()> {
        let file_path = self.data_dir.join("user_settings.json");
        let json_data = serde_json::to_string_pretty(settings)?;
        fs::write(file_path, json_data)?;
        Ok(())
    }

    pub async fn load_user_settings(&self) -> Result<UserSettings> {
        let file_path = self.data_dir.join("user_settings.json");
        if file_path.exists() {
            let json_data = fs::read_to_string(file_path)?;
            let settings: UserSettings = serde_json::from_str(&json_data)?;
            Ok(settings)
        } else {
            Ok(UserSettings::default())
        }
    }

    pub async fn save_task(&self, task: &Task) -> Result<()> {
        let mut tasks = self.load_tasks().await.unwrap_or_default();
        
        // 检查是否是更新现有任务
        if let Some(index) = tasks.iter().position(|t| t.id == task.id) {
            tasks[index] = task.clone();
        } else {
            tasks.push(task.clone());
        }
        
        let file_path = self.data_dir.join("tasks.json");
        let json_data = serde_json::to_string_pretty(&tasks)?;
        fs::write(file_path, json_data)?;
        Ok(())
    }

    pub async fn load_tasks(&self) -> Result<Vec<Task>> {
        let file_path = self.data_dir.join("tasks.json");
        if file_path.exists() {
            let json_data = fs::read_to_string(file_path)?;
            let tasks: Vec<Task> = serde_json::from_str(&json_data)?;
            Ok(tasks)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn delete_task(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.load_tasks().await.unwrap_or_default();
        tasks.retain(|task| task.id != task_id);
        
        let file_path = self.data_dir.join("tasks.json");
        let json_data = serde_json::to_string_pretty(&tasks)?;
        fs::write(file_path, json_data)?;
        Ok(())
    }

    pub async fn update_task_status(&self, task_id: &str, completed: bool) -> Result<()> {
        let mut tasks = self.load_tasks().await.unwrap_or_default();
        
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            task.completed = completed;
            task.updated_at = chrono::Utc::now();
        }
        
        let file_path = self.data_dir.join("tasks.json");
        let json_data = serde_json::to_string_pretty(&tasks)?;
        fs::write(file_path, json_data)?;
        Ok(())
    }

    /// 保存监控结果日志
    pub async fn save_monitoring_result(&self, result: &crate::services::monitor_service::MonitoringResult) -> Result<()> {
        let mut results = self.load_monitoring_results().await.unwrap_or_default();
        results.push(result.clone());
        
        // 只保留最近30天的数据
        let thirty_days_ago = chrono::Utc::now() - chrono::Duration::days(30);
        results.retain(|r| r.timestamp > thirty_days_ago);
        
        let file_path = self.data_dir.join("monitoring_results.json");
        let json_data = serde_json::to_string_pretty(&results)?;
        fs::write(file_path, json_data)?;
        Ok(())
    }

    /// 加载监控结果日志
    pub async fn load_monitoring_results(&self) -> Result<Vec<crate::services::monitor_service::MonitoringResult>> {
        let file_path = self.data_dir.join("monitoring_results.json");
        if file_path.exists() {
            let json_data = fs::read_to_string(file_path)?;
            let results: Vec<crate::services::monitor_service::MonitoringResult> = serde_json::from_str(&json_data)?;
            Ok(results)
        } else {
            Ok(Vec::new())
        }
    }

    /// 获取今日监控统计
    pub async fn get_today_monitoring_stats(&self) -> Result<crate::commands::TodayStats> {
        let results = self.load_monitoring_results().await.unwrap_or_default();
        
        let today = chrono::Utc::now().date_naive();
        let today_results: Vec<_> = results.iter()
            .filter(|r| r.timestamp.date_naive() == today)
            .collect();

        let mut focus_time = 0;
        let mut distract_time = 0;
        let mut severe_distract_time = 0;
        let mut interruption_count = 0;

        // 假设每次监控代表配置的时间间隔（默认3分钟）
        let interval_seconds = 3 * 60; // 3分钟 = 180秒

        for result in &today_results {
            match result.focus_state {
                crate::services::monitor_service::FocusState::Focused => {
                    focus_time += interval_seconds;
                },
                crate::services::monitor_service::FocusState::Distracted => {
                    distract_time += interval_seconds;
                    interruption_count += 1;
                },
                crate::services::monitor_service::FocusState::SeverelyDistracted => {
                    severe_distract_time += interval_seconds;
                    interruption_count += 1;
                },
                _ => {} // Unknown状态不计算
            }
        }

        let total_time = focus_time + distract_time + severe_distract_time;
        let focus_score = if total_time > 0 {
            ((focus_time as f32 / total_time as f32) * 100.0) as u32
        } else {
            0
        };

        Ok(crate::commands::TodayStats {
            total_focus_time: focus_time,
            total_distract_time: distract_time + severe_distract_time,
            focus_score,
            interruption_count,
        })
    }

    /// 保存AI配置
    pub async fn save_ai_config(&self, config: &AIConfig) -> Result<()> {
        let file_path = self.data_dir.join("ai_config.json");
        let json_data = serde_json::to_string_pretty(config)?;
        fs::write(file_path, json_data)?;
        Ok(())
    }

    /// 加载AI配置
    pub async fn load_ai_config(&self) -> Result<AIConfig> {
        let file_path = self.data_dir.join("ai_config.json");
        if file_path.exists() {
            let json_data = fs::read_to_string(file_path)?;
            let config: AIConfig = serde_json::from_str(&json_data)?;
            Ok(config)
        } else {
            Ok(AIConfig::default())
        }
    }

    /// 保存监控配置
    pub async fn save_monitoring_config(&self, config: &crate::services::monitor_service::MonitoringConfig) -> Result<()> {
        let file_path = self.data_dir.join("monitoring_config.json");
        let json_data = serde_json::to_string_pretty(config)?;
        fs::write(file_path, json_data)?;
        Ok(())
    }

    /// 加载监控配置
    pub async fn load_monitoring_config(&self) -> Result<crate::services::monitor_service::MonitoringConfig> {
        let file_path = self.data_dir.join("monitoring_config.json");
        if file_path.exists() {
            let json_data = fs::read_to_string(file_path)?;
            let config: crate::services::monitor_service::MonitoringConfig = serde_json::from_str(&json_data)?;
            Ok(config)
        } else {
            Ok(crate::services::monitor_service::MonitoringConfig::default())
        }
    }

    pub async fn save_focus_session(&self, session: &FocusSession) -> Result<()> {
        let mut sessions = self.load_focus_sessions().await.unwrap_or_default();
        
        // 检查是否是更新现有会话
        if let Some(index) = sessions.iter().position(|s| s.id == session.id) {
            sessions[index] = session.clone();
        } else {
            sessions.push(session.clone());
        }
        
        let file_path = self.data_dir.join("focus_sessions.json");
        let json_data = serde_json::to_string_pretty(&sessions)?;
        fs::write(file_path, json_data)?;
        Ok(())
    }

    pub async fn load_focus_sessions(&self) -> Result<Vec<FocusSession>> {
        let file_path = self.data_dir.join("focus_sessions.json");
        if file_path.exists() {
            let json_data = fs::read_to_string(file_path)?;
            let sessions: Vec<FocusSession> = serde_json::from_str(&json_data)?;
            Ok(sessions)
        } else {
            Ok(Vec::new())
        }
    }

    /// 保存应用活动记录
    pub async fn save_application_activity(&self, _activity: &ApplicationActivity) -> Result<()> {
        // TODO: 实现应用活动记录的持久化
        println!("⚠️ 应用活动记录保存功能待实现");
        Ok(())
    }

    pub async fn load_application_activities(&self) -> Result<Vec<ApplicationActivity>> {
        // TODO: 实现从文件加载应用活动列表
        Ok(Vec::new())
    }

    // ===== 数据清理相关方法 =====

    /// 清理旧的监控结果
    pub async fn cleanup_old_monitoring_results(&self, days_to_keep: u32) -> Result<u32> {
        let results = self.load_monitoring_results().await.unwrap_or_default();
        let original_count = results.len();
        
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days_to_keep as i64);
        let filtered_results: Vec<_> = results.into_iter()
            .filter(|r| r.timestamp > cutoff_date)
            .collect();
        
        let cleaned_count = original_count - filtered_results.len();
        
        if cleaned_count > 0 {
            let file_path = self.data_dir.join("monitoring_results.json");
            let json_data = serde_json::to_string_pretty(&filtered_results)?;
            fs::write(file_path, json_data)?;
            println!("🧹 清理了 {} 条监控记录", cleaned_count);
        }
        
        Ok(cleaned_count as u32)
    }

    /// 清理旧的专注会话
    pub async fn cleanup_old_focus_sessions(&self, days_to_keep: u32) -> Result<u32> {
        let sessions = self.load_focus_sessions().await.unwrap_or_default();
        let original_count = sessions.len();
        
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days_to_keep as i64);
        let filtered_sessions: Vec<_> = sessions.into_iter()
            .filter(|s| {
                if let Some(started_at) = s.started_at {
                    started_at > cutoff_date
                } else {
                    false // 没有开始时间的会话被清理
                }
            })
            .collect();
        
        let cleaned_count = original_count - filtered_sessions.len();
        
        if cleaned_count > 0 {
            let file_path = self.data_dir.join("focus_sessions.json");
            let json_data = serde_json::to_string_pretty(&filtered_sessions)?;
            fs::write(file_path, json_data)?;
            println!("🧹 清理了 {} 个专注会话", cleaned_count);
        }
        
        Ok(cleaned_count as u32)
    }

    /// 清理空任务和重复任务
    pub async fn cleanup_duplicate_tasks(&self) -> Result<u32> {
        let tasks = self.load_tasks().await.unwrap_or_default();
        let original_count = tasks.len();
        
        let mut seen_texts = std::collections::HashSet::new();
        let filtered_tasks: Vec<_> = tasks.into_iter()
            .filter(|task| {
                // 过滤掉空任务
                if task.text.trim().is_empty() {
                    return false;
                }
                
                // 过滤掉重复任务
                seen_texts.insert(task.text.clone())
            })
            .collect();
        
        let cleaned_count = original_count - filtered_tasks.len();
        
        if cleaned_count > 0 {
            let file_path = self.data_dir.join("tasks.json");
            let json_data = serde_json::to_string_pretty(&filtered_tasks)?;
            fs::write(file_path, json_data)?;
            println!("🧹 清理了 {} 个重复/空任务", cleaned_count);
        }
        
        Ok(cleaned_count as u32)
    }

    /// 压缩监控数据
    pub async fn compress_monitoring_data(&self) -> Result<u32> {
        let mut results = self.load_monitoring_results().await.unwrap_or_default();
        let mut compressed_bytes = 0u32;
        
        for result in &mut results {
            // 压缩OCR文本
            if let Some(ref mut ocr_text) = result.ocr_text {
                let original_len = ocr_text.len();
                
                // 移除重复的空白字符
                *ocr_text = ocr_text.split_whitespace().collect::<Vec<_>>().join(" ");
                
                // 如果文本太长，只保留前1000个字符
                if ocr_text.len() > 1000 {
                    ocr_text.truncate(1000);
                    ocr_text.push_str("...[截断]");
                }
                
                compressed_bytes += (original_len - ocr_text.len()) as u32;
            }
            
            // 压缩AI分析结果
            if let Some(ref mut ai_analysis) = result.ai_analysis {
                let original_len = ai_analysis.len();
                
                // 如果AI分析太长，只保留关键部分
                if ai_analysis.len() > 500 {
                    // 尝试保留"状态:"和"分析:"部分
                    if let Some(status_pos) = ai_analysis.find("状态:") {
                        let truncated = if ai_analysis.len() > status_pos + 500 {
                            format!("{}...[截断]", &ai_analysis[..status_pos + 500])
                        } else {
                            ai_analysis.clone()
                        };
                        *ai_analysis = truncated;
                    } else {
                        ai_analysis.truncate(500);
                        ai_analysis.push_str("...[截断]");
                    }
                }
                
                compressed_bytes += (original_len - ai_analysis.len()) as u32;
            }
        }
        
        if compressed_bytes > 0 {
            let file_path = self.data_dir.join("monitoring_results.json");
            let json_data = serde_json::to_string_pretty(&results)?;
            fs::write(file_path, json_data)?;
            println!("🗜️ 压缩监控数据节省了 {} 字节", compressed_bytes);
        }
        
        Ok(compressed_bytes)
    }

    /// 获取存储目录大小
    pub async fn get_storage_size(&self) -> Result<u64> {
        let mut total_size = 0u64;
        
        if let Ok(entries) = fs::read_dir(&self.data_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total_size += metadata.len();
                    }
                }
            }
        }
        
        Ok(total_size)
    }
} 