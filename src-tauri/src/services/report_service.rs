use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, Duration, Timelike};
use std::collections::HashMap;
use anyhow::{Result, anyhow};

use crate::services::storage_service::StorageService;
use crate::services::ai_service::AIService;
use crate::services::monitor_service::{MonitoringResult, FocusState};
use crate::models::FocusSession;

/// 应用使用统计
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppUsageStats {
    pub app_name: String,
    pub total_time_seconds: u32,
    pub focus_time_seconds: u32,
    pub distraction_time_seconds: u32,
    pub switch_count: u32,
}

/// 专注模式统计
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FocusPatternStats {
    pub focus_percentage: f32,
    pub average_focus_duration_minutes: f32,
    pub longest_focus_duration_minutes: u32,
    pub focus_sessions_count: u32,
    pub distraction_interruptions: u32,
}

/// 时间段分析
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeSlotAnalysis {
    pub hour: u8,
    pub focus_percentage: f32,
    pub activity_count: u32,
}

/// 日报告数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct DailyReport {
    pub date: String,
    pub summary: DailyReportSummary,
    pub focus_patterns: FocusPatternStats,
    pub app_usage: Vec<AppUsageStats>,
    pub time_analysis: Vec<TimeSlotAnalysis>,
    pub ai_insights: AIInsights,
    pub recommendations: Vec<String>,
}

/// 日报告摘要
#[derive(Debug, Serialize, Deserialize)]
pub struct DailyReportSummary {
    pub total_monitoring_time_seconds: u32,
    pub focus_time_seconds: u32,
    pub distraction_time_seconds: u32,
    pub focus_score: f32,
    pub productivity_rating: String,
    pub interruption_count: u32,
}

/// 周报告数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct WeeklyReport {
    pub week_start: String,
    pub week_end: String,
    pub summary: WeeklyReportSummary,
    pub daily_trends: Vec<DailyTrendData>,
    pub focus_improvement: FocusImprovementAnalysis,
    pub ai_insights: AIInsights,
    pub weekly_recommendations: Vec<String>,
}

/// 周报告摘要
#[derive(Debug, Serialize, Deserialize)]
pub struct WeeklyReportSummary {
    pub total_focus_time_seconds: u32,
    pub average_daily_focus_score: f32,
    pub best_focus_day: String,
    pub productivity_trend: String,
    pub total_sessions: u32,
}

/// 每日趋势数据
#[derive(Debug, Serialize, Deserialize)]
pub struct DailyTrendData {
    pub date: String,
    pub focus_score: f32,
    pub focus_time_seconds: u32,
    pub session_count: u32,
}

/// 专注改进分析
#[derive(Debug, Serialize, Deserialize)]
pub struct FocusImprovementAnalysis {
    pub improvement_percentage: f32,
    pub best_practices_identified: Vec<String>,
    pub areas_for_improvement: Vec<String>,
}

/// AI洞察
#[derive(Debug, Serialize, Deserialize)]
pub struct AIInsights {
    pub performance_summary: String,
    pub pattern_analysis: String,
    pub behavioral_insights: String,
    pub productivity_suggestions: String,
}

/// 报告服务
pub struct ReportService {
    storage_service: StorageService,
}

impl ReportService {
    pub fn new(storage_service: StorageService) -> Self {
        Self {
            storage_service,
        }
    }

    /// 生成日报告
    pub async fn generate_daily_report(&self, date: &str, ai_service: &AIService) -> Result<DailyReport> {
        println!("📊 开始生成日报告: {}", date);
        
        let target_date = self.parse_date(date)?;
        
        // 1. 获取当日监控数据
        let monitoring_results = self.get_daily_monitoring_data(&target_date).await?;
        println!("📋 获取到 {} 条监控记录", monitoring_results.len());
        
        if monitoring_results.is_empty() {
            return Err(anyhow!("当日无监控数据"));
        }

        // 2. 获取当日专注会话数据
        let focus_sessions = self.get_daily_focus_sessions(&target_date).await?;
        println!("⏱️ 获取到 {} 个专注会话", focus_sessions.len());

        // 3. 数据分析和聚合
        let summary = self.calculate_daily_summary(&monitoring_results, &focus_sessions)?;
        let focus_patterns = self.analyze_focus_patterns(&monitoring_results, &focus_sessions)?;
        let app_usage = self.analyze_app_usage(&monitoring_results)?;
        let time_analysis = self.analyze_time_slots(&monitoring_results)?;

        // 4. 生成AI洞察
        let ai_insights = self.generate_ai_insights(&summary, &focus_patterns, &app_usage, &monitoring_results, ai_service).await?;
        
        // 5. 生成个性化建议
        let recommendations = self.generate_recommendations(&summary, &focus_patterns, &app_usage)?;

        let report = DailyReport {
            date: date.to_string(),
            summary,
            focus_patterns,
            app_usage,
            time_analysis,
            ai_insights,
            recommendations,
        };

        println!("✅ 日报告生成完成");
        Ok(report)
    }

    /// 生成周报告
    pub async fn generate_weekly_report(&self, week_start: &str, ai_service: &AIService) -> Result<WeeklyReport> {
        println!("📊 开始生成周报告: {}", week_start);
        
        let start_date = self.parse_date(week_start)?;
        let end_date = start_date + Duration::days(6);
        
        // 获取整周的数据
        let mut daily_data = Vec::new();
        let mut all_monitoring_results = Vec::new();
        let mut all_focus_sessions = Vec::new();
        
        for i in 0..7 {
            let current_date = start_date + Duration::days(i);
            let date_str = current_date.format("%Y-%m-%d").to_string();
            
            let monitoring_results = self.get_daily_monitoring_data(&current_date).await.unwrap_or_default();
            let focus_sessions = self.get_daily_focus_sessions(&current_date).await.unwrap_or_default();
            
            if !monitoring_results.is_empty() {
                let daily_summary = self.calculate_daily_summary(&monitoring_results, &focus_sessions)?;
                daily_data.push(DailyTrendData {
                    date: date_str,
                    focus_score: daily_summary.focus_score,
                    focus_time_seconds: daily_summary.focus_time_seconds,
                    session_count: focus_sessions.len() as u32,
                });
                
                all_monitoring_results.extend(monitoring_results);
                all_focus_sessions.extend(focus_sessions);
            }
        }

        if daily_data.is_empty() {
            return Err(anyhow!("本周无有效数据"));
        }

        // 计算周摘要
        let summary = self.calculate_weekly_summary(&daily_data)?;
        
        // 分析专注改进情况
        let focus_improvement = self.analyze_focus_improvement(&daily_data)?;
        
        // 生成AI洞察
        let ai_insights = self.generate_weekly_ai_insights(&daily_data, &all_monitoring_results, ai_service).await?;
        
        // 生成周建议
        let weekly_recommendations = self.generate_weekly_recommendations(&summary, &focus_improvement)?;

        let report = WeeklyReport {
            week_start: week_start.to_string(),
            week_end: end_date.format("%Y-%m-%d").to_string(),
            summary,
            daily_trends: daily_data,
            focus_improvement,
            ai_insights,
            weekly_recommendations,
        };

        println!("✅ 周报告生成完成");
        Ok(report)
    }

    /// 解析日期字符串
    fn parse_date(&self, date_str: &str) -> Result<NaiveDate> {
        let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|e| anyhow!("日期格式错误: {}", e))?;
        Ok(naive_date)
    }

    /// 获取指定日期的监控数据
    async fn get_daily_monitoring_data(&self, date: &NaiveDate) -> Result<Vec<MonitoringResult>> {
        let all_results = self.storage_service.load_monitoring_results().await?;
        
        let filtered_results: Vec<MonitoringResult> = all_results
            .into_iter()
            .filter(|result| {
                let result_date = result.timestamp.date_naive();
                result_date == *date
            })
            .collect();
            
        Ok(filtered_results)
    }

    /// 获取指定日期的专注会话数据
    async fn get_daily_focus_sessions(&self, date: &NaiveDate) -> Result<Vec<FocusSession>> {
        let all_sessions = self.storage_service.load_focus_sessions().await?;
        
        let filtered_sessions: Vec<FocusSession> = all_sessions
            .into_iter()
            .filter(|session| {
                if let Some(started_at) = session.started_at {
                    let session_date = started_at.date_naive();
                    session_date == *date
                } else {
                    false
                }
            })
            .collect();
            
        Ok(filtered_sessions)
    }

    /// 计算日摘要统计
    fn calculate_daily_summary(&self, monitoring_results: &[MonitoringResult], _focus_sessions: &[FocusSession]) -> Result<DailyReportSummary> {
        let total_monitoring_time = monitoring_results.len() as u32 * 180; // 假设3分钟间隔
        
        let focus_time = monitoring_results
            .iter()
            .filter(|r| matches!(r.focus_state, FocusState::Focused))
            .count() as u32 * 180;
            
        let distraction_time = monitoring_results
            .iter()
            .filter(|r| matches!(r.focus_state, FocusState::Distracted | FocusState::SeverelyDistracted))
            .count() as u32 * 180;

        let focus_score = if total_monitoring_time > 0 {
            (focus_time as f32 / total_monitoring_time as f32) * 100.0
        } else {
            0.0
        };

        let productivity_rating = match focus_score {
            s if s >= 80.0 => "优秀".to_string(),
            s if s >= 60.0 => "良好".to_string(),
            s if s >= 40.0 => "一般".to_string(),
            _ => "需要改进".to_string(),
        };

        // 计算中断次数（专注状态到分心状态的转换）
        let mut interruption_count = 0u32;
        for window in monitoring_results.windows(2) {
            if matches!(window[0].focus_state, FocusState::Focused) &&
               matches!(window[1].focus_state, FocusState::Distracted | FocusState::SeverelyDistracted) {
                interruption_count += 1;
            }
        }

        Ok(DailyReportSummary {
            total_monitoring_time_seconds: total_monitoring_time,
            focus_time_seconds: focus_time,
            distraction_time_seconds: distraction_time,
            focus_score,
            productivity_rating,
            interruption_count,
        })
    }

    /// 分析专注模式
    fn analyze_focus_patterns(&self, monitoring_results: &[MonitoringResult], focus_sessions: &[FocusSession]) -> Result<FocusPatternStats> {
        let total_time = monitoring_results.len() as f32 * 3.0; // 3分钟间隔
        let focus_time = monitoring_results
            .iter()
            .filter(|r| matches!(r.focus_state, FocusState::Focused))
            .count() as f32 * 3.0;

        let focus_percentage = if total_time > 0.0 {
            (focus_time / total_time) * 100.0
        } else {
            0.0
        };

        // 计算专注持续时长
        let mut focus_durations = Vec::new();
        let mut current_focus_duration = 0u32;
        
        for result in monitoring_results {
            if matches!(result.focus_state, FocusState::Focused) {
                current_focus_duration += 3; // 3分钟
            } else if current_focus_duration > 0 {
                focus_durations.push(current_focus_duration);
                current_focus_duration = 0;
            }
        }
        
        if current_focus_duration > 0 {
            focus_durations.push(current_focus_duration);
        }

        let average_focus_duration = if !focus_durations.is_empty() {
            focus_durations.iter().sum::<u32>() as f32 / focus_durations.len() as f32 / 60.0
        } else {
            0.0
        };

        let longest_focus_duration = focus_durations.iter().max().copied().unwrap_or(0) / 60;

        // 计算干扰次数
        let distraction_interruptions = monitoring_results
            .windows(2)
            .filter(|window| {
                matches!(window[0].focus_state, FocusState::Focused) &&
                matches!(window[1].focus_state, FocusState::Distracted | FocusState::SeverelyDistracted)
            })
            .count() as u32;

        Ok(FocusPatternStats {
            focus_percentage,
            average_focus_duration_minutes: average_focus_duration,
            longest_focus_duration_minutes: longest_focus_duration,
            focus_sessions_count: focus_sessions.len() as u32,
            distraction_interruptions,
        })
    }

    /// 分析应用使用情况
    fn analyze_app_usage(&self, monitoring_results: &[MonitoringResult]) -> Result<Vec<AppUsageStats>> {
        let mut app_stats: HashMap<String, AppUsageStats> = HashMap::new();

        for result in monitoring_results {
            let app_name = result.application_name
                .as_deref()
                .unwrap_or("未知应用")
                .to_string();

            let stats = app_stats.entry(app_name.clone()).or_insert(AppUsageStats {
                app_name,
                total_time_seconds: 0,
                focus_time_seconds: 0,
                distraction_time_seconds: 0,
                switch_count: 0,
            });

            stats.total_time_seconds += 180; // 3分钟间隔

            match result.focus_state {
                FocusState::Focused => stats.focus_time_seconds += 180,
                FocusState::Distracted | FocusState::SeverelyDistracted => stats.distraction_time_seconds += 180,
                _ => {}
            }
        }

        // 计算应用切换次数
        for window in monitoring_results.windows(2) {
            let app1 = window[0].application_name.as_deref().unwrap_or("未知应用");
            let app2 = window[1].application_name.as_deref().unwrap_or("未知应用");
            
            if app1 != app2 {
                if let Some(stats) = app_stats.get_mut(app2) {
                    stats.switch_count += 1;
                }
            }
        }

        let mut result: Vec<AppUsageStats> = app_stats.into_values().collect();
        result.sort_by(|a, b| b.total_time_seconds.cmp(&a.total_time_seconds));
        
        Ok(result)
    }

    /// 分析时间段使用情况
    fn analyze_time_slots(&self, monitoring_results: &[MonitoringResult]) -> Result<Vec<TimeSlotAnalysis>> {
        let mut hour_stats: HashMap<u8, (u32, u32)> = HashMap::new(); // (总次数, 专注次数)

        for result in monitoring_results {
            let hour = result.timestamp.hour() as u8;
            let (total, focused) = hour_stats.entry(hour).or_insert((0, 0));
            
            *total += 1;
            if matches!(result.focus_state, FocusState::Focused) {
                *focused += 1;
            }
        }

        let mut result: Vec<TimeSlotAnalysis> = hour_stats
            .into_iter()
            .map(|(hour, (total, focused))| {
                let focus_percentage = if total > 0 {
                    (focused as f32 / total as f32) * 100.0
                } else {
                    0.0
                };
                
                TimeSlotAnalysis {
                    hour,
                    focus_percentage,
                    activity_count: total,
                }
            })
            .collect();

        result.sort_by_key(|slot| slot.hour);
        Ok(result)
    }

    /// 生成AI洞察
    async fn generate_ai_insights(
        &self,
        summary: &DailyReportSummary,
        focus_patterns: &FocusPatternStats,
        app_usage: &[AppUsageStats],
        monitoring_results: &[MonitoringResult],
        ai_service: &AIService,
    ) -> Result<AIInsights> {
        println!("🤖 开始生成AI洞察...");

        let prompt = self.build_daily_analysis_prompt(summary, focus_patterns, app_usage, monitoring_results);
        
        match ai_service.analyze_content(&prompt, "report").await {
            Ok(ai_response) => {
                println!("✅ AI分析完成");
                Ok(self.parse_ai_insights(&ai_response))
            }
            Err(e) => {
                println!("⚠️ AI分析失败: {}, 使用默认洞察", e);
                Ok(self.generate_default_insights(summary, focus_patterns))
            }
        }
    }

    /// 构建AI分析提示词
    fn build_daily_analysis_prompt(
        &self,
        summary: &DailyReportSummary,
        focus_patterns: &FocusPatternStats,
        app_usage: &[AppUsageStats],
        monitoring_results: &[MonitoringResult],
    ) -> String {
        let top_apps = app_usage
            .iter()
            .take(5)
            .map(|app| format!("{}({}分钟)", app.app_name, app.total_time_seconds / 60))
            .collect::<Vec<_>>()
            .join("、");

        let sample_activities = monitoring_results
            .iter()
            .take(10)
            .filter_map(|r| r.ai_analysis.as_ref())
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"请基于以下专注度监控数据，生成一份专业的个人专注力分析报告。

## 基础数据：
- 总监控时长：{}分钟
- 专注时间：{}分钟 ({:.1}%)
- 分心时间：{}分钟
- 中断次数：{}次
- 平均专注持续时长：{:.1}分钟
- 最长专注时长：{}分钟

## 应用使用情况：
主要使用的应用：{}

## 部分活动记录：
{}

请按以下结构生成分析报告：

**表现总结：**
[总体评价当天的专注表现，突出亮点和问题]

**模式分析：**
[分析专注和分心的模式，识别时间规律]

**行为洞察：**
[深入分析行为特征，找出影响专注的因素]

**改进建议：**
[提供3-5条具体可行的改进建议]

请确保分析客观、专业，建议实用可行。"#,
            summary.total_monitoring_time_seconds / 60,
            summary.focus_time_seconds / 60,
            summary.focus_score,
            summary.distraction_time_seconds / 60,
            summary.interruption_count,
            focus_patterns.average_focus_duration_minutes,
            focus_patterns.longest_focus_duration_minutes,
            if top_apps.is_empty() { "无记录".to_string() } else { top_apps },
            if sample_activities.is_empty() { "无详细记录".to_string() } else { sample_activities }
        )
    }

    /// 解析AI洞察结果
    fn parse_ai_insights(&self, ai_response: &str) -> AIInsights {
        // 简单的文本解析，可以根据需要优化
        let sections: Vec<&str> = ai_response.split("**").collect();
        
        let mut performance_summary = String::new();
        let mut pattern_analysis = String::new();
        let mut behavioral_insights = String::new();
        let mut productivity_suggestions = String::new();

        for window in sections.windows(2) {
            match window[0].trim() {
                "表现总结：" => performance_summary = window[1].trim().to_string(),
                "模式分析：" => pattern_analysis = window[1].trim().to_string(),
                "行为洞察：" => behavioral_insights = window[1].trim().to_string(),
                "改进建议：" => productivity_suggestions = window[1].trim().to_string(),
                _ => {}
            }
        }

        // 如果解析失败，使用整个响应作为总结
        if performance_summary.is_empty() {
            performance_summary = ai_response.to_string();
        }

        AIInsights {
            performance_summary,
            pattern_analysis,
            behavioral_insights,
            productivity_suggestions,
        }
    }

    /// 生成默认洞察（AI不可用时）
    fn generate_default_insights(&self, summary: &DailyReportSummary, focus_patterns: &FocusPatternStats) -> AIInsights {
        let performance_summary = format!(
            "今日专注表现{}。专注时长{}分钟，专注率{:.1}%，发生{}次中断。",
            summary.productivity_rating,
            summary.focus_time_seconds / 60,
            summary.focus_score,
            summary.interruption_count
        );

        let pattern_analysis = format!(
            "平均专注持续时长{:.1}分钟，最长连续专注{}分钟。建议在专注表现好的时间段安排重要工作。",
            focus_patterns.average_focus_duration_minutes,
            focus_patterns.longest_focus_duration_minutes
        );

        AIInsights {
            performance_summary,
            pattern_analysis,
            behavioral_insights: "基于监控数据，建议分析个人专注习惯，识别分心触发因素。".to_string(),
            productivity_suggestions: "建议：1.减少应用切换频率 2.设置专注时间段 3.消除干扰源 4.定期休息恢复专注力".to_string(),
        }
    }

    /// 生成个性化建议
    fn generate_recommendations(&self, summary: &DailyReportSummary, focus_patterns: &FocusPatternStats, app_usage: &[AppUsageStats]) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        // 基于专注分数的建议
        if summary.focus_score < 50.0 {
            recommendations.push("专注度较低，建议减少分心源，创建更好的工作环境".to_string());
        } else if summary.focus_score > 80.0 {
            recommendations.push("专注表现优秀！保持当前的工作节奏和环境设置".to_string());
        }

        // 基于中断次数的建议
        if summary.interruption_count > 10 {
            recommendations.push("中断频率较高，建议关闭不必要的通知，设置专注时间段".to_string());
        }

        // 基于专注持续时长的建议
        if focus_patterns.average_focus_duration_minutes < 15.0 {
            recommendations.push("专注持续时间较短，建议使用番茄工作法，逐步延长专注时间".to_string());
        }

        // 基于应用使用的建议
        if let Some(top_app) = app_usage.first() {
            if top_app.distraction_time_seconds > top_app.focus_time_seconds {
                recommendations.push(format!("在{}应用上分心时间较多，建议限制使用或调整使用方式", top_app.app_name));
            }
        }

        // 基于时间的建议
        if summary.total_monitoring_time_seconds < 4 * 3600 {
            recommendations.push("监控时间较短，建议延长工作时间或提高时间利用效率".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("继续保持良好的专注习惯，可以尝试挑战更高的专注目标".to_string());
        }

        Ok(recommendations)
    }

    /// 计算周摘要
    fn calculate_weekly_summary(&self, daily_data: &[DailyTrendData]) -> Result<WeeklyReportSummary> {
        let total_focus_time_seconds = daily_data.iter().map(|d| d.focus_time_seconds).sum();
        let average_daily_focus_score = daily_data.iter().map(|d| d.focus_score).sum::<f32>() / daily_data.len() as f32;
        let total_sessions = daily_data.iter().map(|d| d.session_count).sum();

        let best_focus_day = daily_data
            .iter()
            .max_by(|a, b| a.focus_score.partial_cmp(&b.focus_score).unwrap())
            .map(|d| d.date.clone())
            .unwrap_or_else(|| "无数据".to_string());

        // 计算趋势
        let productivity_trend = if daily_data.len() >= 2 {
            let first_half_avg = daily_data[..daily_data.len()/2].iter().map(|d| d.focus_score).sum::<f32>() / (daily_data.len()/2) as f32;
            let second_half_avg = daily_data[daily_data.len()/2..].iter().map(|d| d.focus_score).sum::<f32>() / (daily_data.len() - daily_data.len()/2) as f32;
            
            if second_half_avg > first_half_avg + 5.0 {
                "上升".to_string()
            } else if second_half_avg < first_half_avg - 5.0 {
                "下降".to_string()
            } else {
                "稳定".to_string()
            }
        } else {
            "数据不足".to_string()
        };

        Ok(WeeklyReportSummary {
            total_focus_time_seconds,
            average_daily_focus_score,
            best_focus_day,
            productivity_trend,
            total_sessions,
        })
    }

    /// 分析专注改进情况
    fn analyze_focus_improvement(&self, daily_data: &[DailyTrendData]) -> Result<FocusImprovementAnalysis> {
        if daily_data.len() < 2 {
            return Ok(FocusImprovementAnalysis {
                improvement_percentage: 0.0,
                best_practices_identified: vec!["数据不足，无法分析".to_string()],
                areas_for_improvement: vec!["建议连续使用一周后查看改进分析".to_string()],
            });
        }

        let first_score = daily_data[0].focus_score;
        let last_score = daily_data.last().unwrap().focus_score;
        let improvement_percentage = ((last_score - first_score) / first_score) * 100.0;

        let best_practices = vec![
            "保持稳定的作息时间".to_string(),
            "创建无干扰的工作环境".to_string(),
            "合理安排休息时间".to_string(),
        ];

        let areas_for_improvement = if improvement_percentage < 0.0 {
            vec![
                "分析最近的分心原因".to_string(),
                "调整工作环境和工具设置".to_string(),
                "重新评估任务优先级".to_string(),
            ]
        } else {
            vec![
                "继续保持良好习惯".to_string(),
                "尝试挑战更高的专注目标".to_string(),
                "分享成功经验".to_string(),
            ]
        };

        Ok(FocusImprovementAnalysis {
            improvement_percentage,
            best_practices_identified: best_practices,
            areas_for_improvement,
        })
    }

    /// 生成周AI洞察
    async fn generate_weekly_ai_insights(
        &self,
        daily_data: &[DailyTrendData],
        monitoring_results: &[MonitoringResult],
        ai_service: &AIService,
    ) -> Result<AIInsights> {
        let prompt = self.build_weekly_analysis_prompt(daily_data, monitoring_results);
        
        match ai_service.analyze_content(&prompt, "report").await {
            Ok(ai_response) => Ok(self.parse_ai_insights(&ai_response)),
            Err(_) => Ok(self.generate_default_weekly_insights(daily_data)),
        }
    }

    /// 构建周分析提示词
    fn build_weekly_analysis_prompt(&self, daily_data: &[DailyTrendData], _monitoring_results: &[MonitoringResult]) -> String {
        let daily_summary = daily_data
            .iter()
            .map(|d| format!("{}: 专注率{:.1}%, 专注时长{}分钟", d.date, d.focus_score, d.focus_time_seconds / 60))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"请基于以下一周的专注度数据，生成周度专注力分析报告：

## 每日数据：
{}

请分析：
1. 本周专注表现的整体趋势
2. 最佳和最差表现日的原因分析
3. 周度专注模式和规律
4. 下周的改进建议

请保持专业客观的分析风格。"#,
            daily_summary
        )
    }

    /// 生成默认周洞察
    fn generate_default_weekly_insights(&self, daily_data: &[DailyTrendData]) -> AIInsights {
        let avg_score = daily_data.iter().map(|d| d.focus_score).sum::<f32>() / daily_data.len() as f32;
        let total_time = daily_data.iter().map(|d| d.focus_time_seconds).sum::<u32>() / 3600;

        AIInsights {
            performance_summary: format!("本周平均专注率{:.1}%，总专注时长{}小时", avg_score, total_time),
            pattern_analysis: "建议分析周度专注模式，识别高效时间段".to_string(),
            behavioral_insights: "通过连续监控发现个人专注规律".to_string(),
            productivity_suggestions: "基于周度数据优化工作安排和时间管理".to_string(),
        }
    }

    /// 生成周建议
    fn generate_weekly_recommendations(&self, summary: &WeeklyReportSummary, _improvement: &FocusImprovementAnalysis) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        if summary.average_daily_focus_score < 60.0 {
            recommendations.push("本周专注度整体偏低，建议重新评估工作环境和习惯".to_string());
        }

        if summary.productivity_trend == "下降" {
            recommendations.push("专注度呈下降趋势，建议分析原因并及时调整".to_string());
        } else if summary.productivity_trend == "上升" {
            recommendations.push("专注度持续改善，继续保持当前的优秀习惯".to_string());
        }

        recommendations.push(format!("以{}为标杆，分析高效日的成功因素", summary.best_focus_day));
        recommendations.push("建议设定下周的专注度目标，持续改进".to_string());

        Ok(recommendations)
    }
}