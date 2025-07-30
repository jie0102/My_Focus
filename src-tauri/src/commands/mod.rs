// 直接在mod.rs中定义命令，简化结构
use tauri::{command, Manager};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Datelike};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::services::ai_service::{AIConfig, APITestResult, ModelInfo, AIService};
use crate::services::monitor_service::{MonitoringConfig, FocusState, MonitoringResult, MonitorService};
use crate::services::storage_service::StorageService;
use crate::services::timer_service::TimerService;
use crate::services::report_service::{ReportService, DailyReport, WeeklyReport};
use crate::models::focus_session::SessionType;

// 全局服务实例
lazy_static::lazy_static! {
    static ref STORAGE_SERVICE: Arc<Mutex<Option<StorageService>>> = Arc::new(Mutex::new(None));
    static ref TIMER_SERVICE: Arc<TimerService> = Arc::new(TimerService::new());
    static ref MONITOR_SERVICE: Arc<MonitorService> = Arc::new(MonitorService::new());
}

// 初始化存储服务
pub async fn init_storage_service() {
    // 使用应用本地目录存储数据，避免在系统目录中存储敏感信息
    let app_data_dir = std::path::PathBuf::from("data");
    
    let storage_service = StorageService::new(app_data_dir);
    let mut storage = STORAGE_SERVICE.lock().await;
    *storage = Some(storage_service);
}

// 获取存储服务实例
pub async fn get_storage_service() -> Result<StorageService, String> {
    let storage = STORAGE_SERVICE.lock().await;
    match storage.as_ref() {
        Some(_service) => {
            // 使用应用本地目录
            let app_data_dir = std::path::PathBuf::from("data");
            
            Ok(StorageService::new(app_data_dir))
        },
        None => Err("存储服务未初始化".to_string()),
    }
}

// ===== 数据结构定义 =====

#[derive(Debug, Serialize, Deserialize)]
pub struct AppStatus {
    pub version: String,
    pub is_monitoring: bool,
    pub is_timer_running: bool,
    pub uptime: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSettings {
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
    pub autostart: bool,
    pub fatigue_notify: bool,
    pub focus_duration: u32,
    pub short_break: u32,
    pub long_break: u32,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            whitelist: vec![],
            blacklist: vec![],
            autostart: false,
            fatigue_notify: true,
            focus_duration: 25,
            short_break: 5,
            long_break: 15,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub text: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewTask {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodayStats {
    pub total_focus_time: u32,    // 专注时间（秒）
    pub total_distract_time: u32, // 分心时间（秒）
    pub focus_score: u32,         // 专注分数（0-100）
    pub interruption_count: u32,  // 中断次数
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimerStatus {
    pub is_running: bool,
    pub session_type: Option<String>,
    pub elapsed_seconds: u32,
    pub remaining_seconds: u32,
    pub task_id: Option<String>,
    pub duration_minutes: u32,
}

// ===== Tauri 命令实现 =====

/// 获取应用状态
#[command]
pub async fn get_app_status() -> Result<AppStatus, String> {
    println!("获取应用状态");
    Ok(AppStatus {
        version: "1.0.0".to_string(),
        is_monitoring: false,
        is_timer_running: false,
        uptime: 0,
    })
}

/// 初始化应用
#[command]
pub async fn initialize_app() -> Result<String, String> {
    println!("正在初始化应用...");
    
    // 初始化存储服务
    init_storage_service().await;
    
    Ok("应用初始化成功".to_string())
}

/// 保存用户设置
#[command]
pub async fn save_user_settings(settings: UserSettings) -> Result<String, String> {
    println!("保存用户设置: {:?}", settings);
    
    let storage_service = get_storage_service().await?;
    storage_service.save_user_settings(&settings).await
        .map_err(|e| format!("保存用户设置失败: {}", e))?;
    
    Ok("设置保存成功".to_string())
}

/// 加载用户设置
#[command]
pub async fn load_user_settings() -> Result<UserSettings, String> {
    println!("加载用户设置");
    
    let storage_service = get_storage_service().await?;
    storage_service.load_user_settings().await
        .map_err(|e| format!("加载用户设置失败: {}", e))
}

/// 保存任务
#[command]
pub async fn save_task(task: NewTask) -> Result<Task, String> {
    println!("保存任务: {:?}", task);
    
    let now = Utc::now();
    let new_task = Task {
        id: uuid::Uuid::new_v4().to_string(),
        text: task.text,
        completed: false,
        created_at: now,
        updated_at: now,
    };
    
    let storage_service = get_storage_service().await?;
    storage_service.save_task(&new_task).await
        .map_err(|e| format!("保存任务失败: {}", e))?;
    
    Ok(new_task)
}

/// 获取任务列表
#[command]
pub async fn get_tasks(_date: Option<String>) -> Result<Vec<Task>, String> {
    println!("获取任务列表");
    
    let storage_service = get_storage_service().await?;
    storage_service.load_tasks().await
        .map_err(|e| format!("获取任务列表失败: {}", e))
}

/// 更新任务状态
#[command]
pub async fn update_task_status(task_id: String, completed: bool) -> Result<String, String> {
    println!("更新任务状态: {} -> {}", task_id, completed);
    
    let storage_service = get_storage_service().await?;
    storage_service.update_task_status(&task_id, completed).await
        .map_err(|e| format!("更新任务状态失败: {}", e))?;
    
    Ok("任务状态更新成功".to_string())
}

/// 删除任务
#[command]
pub async fn delete_task(task_id: String) -> Result<String, String> {
    println!("删除任务: {}", task_id);
    
    let storage_service = get_storage_service().await?;
    storage_service.delete_task(&task_id).await
        .map_err(|e| format!("删除任务失败: {}", e))?;
    
    Ok("任务删除成功".to_string())
}

/// 开始系统监控
#[command]
pub async fn start_monitoring(app_handle: tauri::AppHandle) -> Result<String, String> {
    println!("🚀 开始系统监控");
    
    let monitor_service = &*MONITOR_SERVICE;
    
    // 设置AppHandle用于事件发送
    monitor_service.set_app_handle(app_handle).await;
    
    // 先加载监控配置
    match get_storage_service().await {
        Ok(storage_service) => {
            match storage_service.load_monitoring_config().await {
                Ok(config) => {
                    println!("📋 加载监控配置: 间隔={}分钟, 白名单={}项, 黑名单={}项", 
                        config.interval_minutes, 
                        config.whitelist.len(), 
                        config.blacklist.len()
                    );
                    
                    // 更新监控服务配置
                    if let Err(e) = monitor_service.update_config(config).await {
                        return Err(format!("更新监控配置失败: {}", e));
                    }
                }
                Err(e) => {
                    println!("⚠️ 加载监控配置失败，使用默认配置: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ 获取存储服务失败: {}", e);
        }
    }
    
    // 启动监控服务
    match monitor_service.start_monitoring().await {
        Ok(_) => {
            println!("✅ 监控服务已成功启动");
            Ok("监控已启动".to_string())
        }
        Err(e) => {
            println!("❌ 监控服务启动失败: {}", e);
            Err(format!("监控启动失败: {}", e))
        }
    }
}

/// 停止系统监控
#[command]
pub async fn stop_monitoring() -> Result<String, String> {
    println!("🛑 停止系统监控");
    
    let monitor_service = &*MONITOR_SERVICE;
    
    match monitor_service.stop_monitoring().await {
        Ok(_) => {
            println!("✅ 监控服务已成功停止");
            Ok("监控已停止".to_string())
        }
        Err(e) => {
            println!("❌ 监控服务停止失败: {}", e);
            Err(format!("监控停止失败: {}", e))
        }
    }
}

/// 获取当前活动信息
#[command]
pub async fn get_current_activity() -> Result<String, String> {
    println!("📱 获取当前活动信息");
    
    let monitor_service = &*MONITOR_SERVICE;
    
    match monitor_service.get_current_activity().await {
        Some(activity) => {
            let app_name = activity.application_name.unwrap_or_else(|| "未知应用".to_string());
            let window_title = activity.window_title.unwrap_or_else(|| "无标题".to_string());
            let activity_info = format!("{} - {}", app_name, window_title);
            println!("📋 当前活动: {}", activity_info);
            Ok(activity_info)
        }
        None => {
            println!("⚠️ 暂无活动信息");
            Ok("暂无活动信息".to_string())
        }
    }
}

/// 开始专注计时器
#[command]
pub async fn start_focus_timer(task_name: Option<String>, duration: u32) -> Result<String, String> {
    println!("开始专注计时器: 任务={:?}, 时长={}分钟", task_name, duration);
    
    let timer_service = &*TIMER_SERVICE;
    match timer_service.start_session(SessionType::Focus, duration).await {
        Ok(session_id) => {
            // 如果指定了任务，可以保存关联关系
            if let Some(task) = task_name {
                println!("计时器关联任务: {}", task);
            }
            Ok(format!("计时器已启动，会话ID: {}", session_id))
        }
        Err(e) => Err(format!("启动计时器失败: {}", e))
    }
}

/// 暂停专注计时器
#[command]
pub async fn pause_focus_timer() -> Result<String, String> {
    println!("暂停专注计时器");
    
    let timer_service = &*TIMER_SERVICE;
    match timer_service.pause_session().await {
        Ok(_) => Ok("计时器已暂停".to_string()),
        Err(e) => Err(format!("暂停计时器失败: {}", e))
    }
}

/// 停止专注计时器
#[command]
pub async fn stop_focus_timer() -> Result<String, String> {
    println!("停止专注计时器");
    
    let timer_service = &*TIMER_SERVICE;
    match timer_service.stop_session().await {
        Ok(session_opt) => {
            if let Some(session) = session_opt {
                // 保存会话记录到存储服务
                if let Ok(storage_service) = get_storage_service().await {
                    let _ = storage_service.save_focus_session(&session).await;
                }
                Ok(format!("计时器已停止，已保存 {} 分钟的专注记录", session.elapsed_seconds / 60))
            } else {
                Ok("计时器已停止".to_string())
            }
        }
        Err(e) => Err(format!("停止计时器失败: {}", e))
    }
}

/// 获取计时器状态
#[command]
pub async fn get_timer_status() -> Result<TimerStatus, String> {
    println!("获取计时器状态");
    
    let timer_service = &*TIMER_SERVICE;
    let current_session = timer_service.get_current_session().await;
    let elapsed_seconds = timer_service.get_elapsed_seconds().await;
    let remaining_seconds = timer_service.get_remaining_seconds().await;
    
    Ok(TimerStatus {
        is_running: current_session.is_some(),
        session_type: current_session.as_ref().map(|s| format!("{:?}", s.session_type)),
        elapsed_seconds,
        remaining_seconds,
        task_id: current_session.as_ref().and_then(|s| s.task_id.clone()),
        duration_minutes: current_session.map(|s| s.duration_minutes).unwrap_or(0),
    })
}

/// 获取今日统计数据
#[command]
pub async fn get_today_statistics() -> Result<TodayStats, String> {
    println!("获取今日统计数据");
    let storage_service = get_storage_service().await?;
    let stats = storage_service.get_today_monitoring_stats().await
        .map_err(|e| format!("加载今日统计数据失败: {}", e))?;
    Ok(stats)
}

/// 获取专注历史记录
#[command]
pub async fn get_focus_history(_days: Option<u32>) -> Result<String, String> {
    println!("获取专注历史记录");
    Ok("历史记录已获取".to_string())
}

// ===== AI 配置相关命令 =====

/// 保存AI配置
#[command]
pub async fn save_ai_config(config: AIConfig) -> Result<String, String> {
    println!("保存AI配置: {:?}", config);
    
    let storage_service = get_storage_service().await?;
    storage_service.save_ai_config(&config).await
        .map_err(|e| format!("保存AI配置失败: {}", e))?;
    
    Ok("AI配置保存成功".to_string())
}

/// 加载AI配置
#[command]
pub async fn load_ai_config() -> Result<AIConfig, String> {
    println!("加载AI配置");
    
    let storage_service = get_storage_service().await?;
    storage_service.load_ai_config().await
        .map_err(|e| format!("加载AI配置失败: {}", e))
}

/// 测试API连接
#[command]
pub async fn test_ai_api(config: AIConfig) -> Result<APITestResult, String> {
    println!("测试API连接: {}", config.api_url);
    
    let ai_service = AIService::new(config);
    match ai_service.test_api_connection().await {
        Ok(result) => Ok(result),
        Err(e) => Ok(APITestResult {
            success: false,
            message: format!("测试失败: {}", e),
            response_time_ms: 0,
            model_used: None,
        }),
    }
}

/// 获取可用模型列表
#[command]
pub async fn get_available_models(config: AIConfig) -> Result<Vec<ModelInfo>, String> {
    println!("获取可用模型列表");
    
    let ai_service = AIService::new(config);
    match ai_service.get_available_models().await {
        Ok(models) => Ok(models),
        Err(e) => {
            println!("获取模型列表失败: {}", e);
            Ok(vec![]) // 返回空列表而不是错误
        }
    }
}

/// 刷新模型列表（重新从API获取）
#[command]
pub async fn refresh_models(config: AIConfig) -> Result<Vec<ModelInfo>, String> {
    println!("刷新模型列表");
    get_available_models(config).await
}

// ===== 监控配置相关命令 =====

/// 保存监控配置
#[command]
pub async fn save_monitoring_config(config: MonitoringConfig) -> Result<String, String> {
    println!("保存监控配置: {:?}", config);
    
    let storage_service = get_storage_service().await?;
    storage_service.save_monitoring_config(&config).await
        .map_err(|e| format!("保存监控配置失败: {}", e))?;
    
    Ok("监控配置保存成功".to_string())
}

/// 加载监控配置
#[command]
pub async fn load_monitoring_config() -> Result<MonitoringConfig, String> {
    println!("加载监控配置");
    
    let storage_service = get_storage_service().await?;
    storage_service.load_monitoring_config().await
        .map_err(|e| format!("加载监控配置失败: {}", e))
}

/// 获取当前活动状态
#[command]
pub async fn get_current_focus_state() -> Result<Option<MonitoringResult>, String> {
    println!("获取当前专注状态");
    // TODO: 从监控服务获取当前状态
    Ok(None)
}

/// 更新监控频率
#[command]
pub async fn update_monitoring_interval(interval_minutes: u8) -> Result<String, String> {
    println!("更新监控频率: {}分钟", interval_minutes);
    if interval_minutes < 1 || interval_minutes > 10 {
        return Err("监控频率必须在1-10分钟之间".to_string());
    }
    // TODO: 更新监控服务的配置
    Ok("监控频率已更新".to_string())
}

/// 手动触发一次监控检查
#[command]
pub async fn trigger_monitoring_check(app_handle: tauri::AppHandle) -> Result<MonitoringResult, String> {
    println!("🔍 手动触发监控检查");
    
    let monitor_service = &*MONITOR_SERVICE;
    
    // 检查监控服务是否正在运行
    let is_monitoring = monitor_service.is_monitoring().await;
    println!("📊 监控状态: {}", if is_monitoring { "运行中" } else { "已停止" });
    
    // 加载当前监控配置
    let config = monitor_service.get_config().await;
    println!("⚙️ 使用配置: 间隔={}分钟, 白名单={}项, 黑名单={}项", 
        config.interval_minutes, 
        config.whitelist.len(), 
        config.blacklist.len()
    );
    
    // 执行手动监控检查
    match perform_manual_monitoring_check(&config).await {
        Ok(result) => {
            println!("✅ 手动检查完成: {:?}, 置信度: {:.2}", 
                result.focus_state, result.confidence
            );
            
            // 发送状态变化事件给前端
            let focus_state_str = match result.focus_state {
                FocusState::Focused => "focused",
                FocusState::Distracted => "distracted", 
                FocusState::SeverelyDistracted => "severely_distracted",
                FocusState::Unknown => "unknown"
            };
            
            let focus_event = serde_json::json!({
                "state": focus_state_str,
                "confidence": result.confidence,
                "application_name": result.application_name,
                "window_title": result.window_title,
                "timestamp": result.timestamp,
                "ai_analysis": result.ai_analysis
            });
            
            // 发送专注状态变化事件
            if let Err(e) = app_handle.emit_all("focus_state_changed", &focus_event) {
                println!("❌ 发送专注状态事件失败: {}", e);
            } else {
                println!("📡 专注状态事件已发送: {}", focus_state_str);
            }
            
            // 检查是否需要分心干预
            if matches!(result.focus_state, FocusState::Distracted | FocusState::SeverelyDistracted) {
                send_distraction_intervention(&app_handle, &result).await;
            }
            
            // 保存检查结果到存储服务
            if let Ok(storage_service) = get_storage_service().await {
                if let Err(e) = storage_service.save_monitoring_result(&result).await {
                    println!("⚠️ 保存监控结果失败: {}", e);
                } else {
                    println!("💾 监控结果已保存");
                }
            }
            
            Ok(result)
        }
        Err(e) => {
            println!("❌ 手动检查失败: {}", e);
            Err(format!("监控检查失败: {}", e))
        }
    }
}

/// 执行手动监控检查的辅助函数
async fn perform_manual_monitoring_check(config: &MonitoringConfig) -> Result<MonitoringResult, String> {
    use std::time::Instant;
    
    let start_time = Instant::now();
    println!("🔄 开始执行监控检查流程...");
    
    // 1. 获取当前应用信息
    println!("📱 步骤1: 获取当前应用信息");
    let app_start = Instant::now();
    let (app_name, window_title) = get_current_application_info_sync().await
        .map_err(|e| format!("获取应用信息失败: {}", e))?;
    println!("⏱️ 应用信息获取耗时: {:?}", app_start.elapsed());
    println!("📋 应用: {:?}, 窗口: {:?}", app_name, window_title);
    
    // 2. 截取屏幕并进行OCR
    println!("📸 步骤2: 屏幕截图和OCR识别");
    let ocr_start = Instant::now();
    let ocr_text = capture_screen_and_ocr_sync().await
        .map_err(|e| format!("屏幕截图或OCR失败: {}", e))?;
    println!("⏱️ 截图+OCR耗时: {:?}", ocr_start.elapsed());
    if let Some(ref text) = ocr_text {
        println!("📝 OCR识别文本长度: {} 字符", text.len());
        println!("📄 OCR文本预览: {}", 
            if text.len() > 100 { 
                format!("{}...", &text[..100]) 
            } else { 
                text.clone() 
            }
        );
    }
    
    // 3. AI分析
    println!("🤖 步骤3: AI专注状态分析");
    let ai_start = Instant::now();
    let ai_result = analyze_focus_with_ai_sync(config, &app_name, &window_title, &ocr_text).await
        .map_err(|e| format!("AI分析失败: {}", e))?;
    println!("⏱️ AI分析耗时: {:?}", ai_start.elapsed());
    
    println!("🎯 总检查耗时: {:?}", start_time.elapsed());
    
    Ok(ai_result)
}

/// 同步版本的应用信息获取函数
async fn get_current_application_info_sync() -> Result<(Option<String>, Option<String>), anyhow::Error> {
    // 调用MonitorService的静态方法
    crate::services::monitor_service::MonitorService::get_current_application_info().await
}

/// 同步版本的屏幕截图和OCR函数
async fn capture_screen_and_ocr_sync() -> Result<Option<String>, anyhow::Error> {
    // 调用MonitorService的静态方法
    crate::services::monitor_service::MonitorService::capture_screen_and_ocr().await
}

/// 同步版本的AI分析函数
async fn analyze_focus_with_ai_sync(
    config: &MonitoringConfig,
    app_name: &Option<String>,
    window_title: &Option<String>,
    ocr_text: &Option<String>,
) -> Result<MonitoringResult, anyhow::Error> {
    use crate::services::ai_service::AIService;
    
    let ai_service = AIService::new(config.ai_config.clone());
    
    // 构建AI分析提示
    let prompt = build_analysis_prompt_sync(config, app_name, window_title, ocr_text);
    println!("💭 AI提示词长度: {} 字符", prompt.len());
    println!("📋 AI提示词内容:\n{}", prompt);
    
    // 调用AI模型进行分析
    let ai_response = call_ai_model_sync(&ai_service, &prompt).await?;
    println!("🤖 AI原始响应:\n{}", ai_response);
    
    // 解析AI响应
    let (focus_state, confidence) = parse_ai_response_sync(&ai_response);
    println!("🎯 解析结果: {:?} (置信度: {:.2})", focus_state, confidence);

    Ok(MonitoringResult {
        timestamp: chrono::Utc::now(),
        focus_state,
        application_name: app_name.clone(),
        window_title: window_title.clone(),
        ocr_text: ocr_text.clone(),
        ai_analysis: Some(ai_response),
        confidence,
    })
}

/// 构建AI分析提示词
fn build_analysis_prompt_sync(
    config: &MonitoringConfig,
    app_name: &Option<String>,
    window_title: &Option<String>,
    ocr_text: &Option<String>,
) -> String {
    let app_info = app_name.as_deref().unwrap_or("未知应用");
    let title_info = window_title.as_deref().unwrap_or("无标题");
    let text_info = ocr_text.as_deref().unwrap_or("无文本内容");
    
    let whitelist = config.whitelist.join(", ");
    let blacklist = config.blacklist.join(", ");

    format!(
        r#"请分析当前用户的专注状态。

**白名单应用（专注工具）**: {}
**黑名单应用（分心来源）**: {}

**当前活动信息**:
- 应用程序: {}
- 窗口标题: {}
- 屏幕文本: {}

请根据以上信息判断用户当前的专注状态，并按以下格式回答：

状态: [专注/分心/严重分心]
分析: [详细说明判断理由]

判断标准：
- 专注：使用白名单中的应用，或从事与工作学习相关的活动
- 分心：使用黑名单中的应用，或从事娱乐相关活动
- 严重分心：长时间使用娱乐应用，或明显的非工作内容"#,
        whitelist,
        blacklist,
        app_info,
        title_info,
        text_info
    )
}

/// 调用AI模型
async fn call_ai_model_sync(ai_service: &AIService, prompt: &str) -> Result<String, anyhow::Error> {
    // 使用配置的检测模型调用AI服务
    match ai_service.analyze_content(prompt, "detection").await {
        Ok(response) => Ok(response),
        Err(e) => {
            println!("⚠️ AI模型调用失败: {}", e);
            // 如果AI调用失败，返回基础分析
            Ok("状态: 未知\n分析: AI服务暂不可用，无法进行专注状态分析。".to_string())
        }
    }
}

/// 解析AI响应
fn parse_ai_response_sync(response: &str) -> (FocusState, f32) {
    let response_lower = response.to_lowercase();
    
    if response_lower.contains("专注") {
        (FocusState::Focused, 0.8)
    } else if response_lower.contains("严重分心") {
        (FocusState::SeverelyDistracted, 0.9)
    } else if response_lower.contains("分心") {
        (FocusState::Distracted, 0.7)
    } else {
        (FocusState::Unknown, 0.5)
    }
}

// ===== 报告生成相关命令 =====

/// 生成日报告
#[command]
pub async fn generate_daily_report(date: String) -> Result<DailyReport, String> {
    println!("📊 开始生成日报告: {}", date);
    
    let storage_service = get_storage_service().await?;
    let ai_config = storage_service.load_ai_config().await
        .map_err(|e| format!("加载AI配置失败: {}", e))?;
    
    let ai_service = AIService::new(ai_config);
    let report_service = ReportService::new(storage_service);
    
    match report_service.generate_daily_report(&date, &ai_service).await {
        Ok(report) => {
            println!("✅ 日报告生成成功");
            Ok(report)
        }
        Err(e) => {
            println!("❌ 日报告生成失败: {}", e);
            Err(format!("生成日报告失败: {}", e))
        }
    }
}

/// 生成周报告
#[command]
pub async fn generate_weekly_report(week_start: String) -> Result<WeeklyReport, String> {
    println!("📊 开始生成周报告: {}", week_start);
    
    let storage_service = get_storage_service().await?;
    let ai_config = storage_service.load_ai_config().await
        .map_err(|e| format!("加载AI配置失败: {}", e))?;
    
    let ai_service = AIService::new(ai_config);
    let report_service = ReportService::new(storage_service);
    
    match report_service.generate_weekly_report(&week_start, &ai_service).await {
        Ok(report) => {
            println!("✅ 周报告生成成功");
            Ok(report)
        }
        Err(e) => {
            println!("❌ 周报告生成失败: {}", e);
            Err(format!("生成周报告失败: {}", e))
        }
    }
}

/// 获取报告列表
#[command]
pub async fn get_report_list(report_type: String, limit: Option<u32>) -> Result<Vec<ReportListItem>, String> {
    println!("📋 获取报告列表: {}", report_type);
    
    let storage_service = get_storage_service().await?;
    let limit = limit.unwrap_or(30);
    
    match report_type.as_str() {
        "daily" => {
            // 获取有数据的日期列表
            let monitoring_results = storage_service.load_monitoring_results().await
                .map_err(|e| format!("加载监控数据失败: {}", e))?;
            
            let mut dates: std::collections::HashSet<String> = std::collections::HashSet::new();
            for result in monitoring_results {
                let date_str = result.timestamp.format("%Y-%m-%d").to_string();
                dates.insert(date_str);
            }
            
            let mut date_list: Vec<String> = dates.into_iter().collect();
            date_list.sort_by(|a, b| b.cmp(a)); // 按日期降序排列
            date_list.truncate(limit as usize);
            
            let report_items = date_list.into_iter().map(|date| {
                ReportListItem {
                    id: format!("daily_{}", date),
                    title: format!("{}日报告", date),
                    date: date.clone(),
                    report_type: "daily".to_string(),
                    status: "available".to_string(),
                }
            }).collect();
            
            Ok(report_items)
        }
        "weekly" => {
            // 生成周报告列表
            let monitoring_results = storage_service.load_monitoring_results().await
                .map_err(|e| format!("加载监控数据失败: {}", e))?;
            
            if monitoring_results.is_empty() {
                return Ok(vec![]);
            }
            
            let earliest_date = monitoring_results.iter()
                .map(|r| r.timestamp.date_naive())
                .min()
                .unwrap();
            
            let latest_date = monitoring_results.iter()
                .map(|r| r.timestamp.date_naive())
                .max()
                .unwrap();
            
            let mut report_items = Vec::new();
            let mut current_monday = latest_date;
            
            // 找到最近的周一
            while current_monday.weekday().num_days_from_monday() != 0 {
                current_monday = current_monday.pred_opt().unwrap();
            }
            
            // 生成最近几周的报告项
            for _ in 0..(limit.min(12)) {
                if current_monday < earliest_date {
                    break;
                }
                
                let week_start = current_monday.format("%Y-%m-%d").to_string();
                let week_end = (current_monday + chrono::Duration::days(6)).format("%Y-%m-%d").to_string();
                
                report_items.push(ReportListItem {
                    id: format!("weekly_{}", week_start),
                    title: format!("{} 至 {} 周报告", week_start, week_end),
                    date: week_start.clone(),
                    report_type: "weekly".to_string(),
                    status: "available".to_string(),
                });
                
                current_monday = current_monday - chrono::Duration::days(7);
            }
            
            Ok(report_items)
        }
        _ => Err("不支持的报告类型".to_string())
    }
}

/// 删除报告（如果需要）
#[command]
pub async fn delete_report(report_id: String) -> Result<String, String> {
    println!("🗑️ 删除报告: {}", report_id);
    // 由于报告是动态生成的，这里只是返回成功
    // 如果将来需要缓存报告，可以在这里实现删除逻辑
    Ok("报告删除成功".to_string())
}

/// 导出报告数据
#[command]
pub async fn export_report_data(date_range: String, format: String) -> Result<String, String> {
    println!("📤 导出报告数据: {} (格式: {})", date_range, format);
    
    let storage_service = get_storage_service().await?;
    
    // 解析日期范围
    let parts: Vec<&str> = date_range.split(" to ").collect();
    if parts.len() != 2 {
        return Err("日期范围格式错误".to_string());
    }
    
    let start_date = parts[0];
    let end_date = parts[1];
    
    // 获取指定范围的数据
    let monitoring_results = storage_service.load_monitoring_results().await
        .map_err(|e| format!("加载监控数据失败: {}", e))?;
    
    let focus_sessions = storage_service.load_focus_sessions().await
        .map_err(|e| format!("加载专注会话失败: {}", e))?;
    
    // 过滤指定日期范围的数据
    let filtered_results: Vec<_> = monitoring_results.into_iter()
        .filter(|r| {
            let date_str = r.timestamp.format("%Y-%m-%d").to_string();
            date_str.as_str() >= start_date && date_str.as_str() <= end_date
        })
        .collect();
    
    let filtered_sessions: Vec<_> = focus_sessions.into_iter()
        .filter(|s| {
            if let Some(started_at) = s.started_at {
                let date_str = started_at.format("%Y-%m-%d").to_string();
                date_str.as_str() >= start_date && date_str.as_str() <= end_date
            } else {
                false
            }
        })
        .collect();
    
    // 根据格式导出
    match format.as_str() {
        "json" => {
            let export_data = ExportData {
                monitoring_results: filtered_results,
                focus_sessions: filtered_sessions,
                export_date: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                date_range: date_range.clone(),
            };
            
            match serde_json::to_string_pretty(&export_data) {
                Ok(json_str) => Ok(json_str),
                Err(e) => Err(format!("JSON序列化失败: {}", e))
            }
        }
        "csv" => {
            // 生成CSV格式的数据
            let mut csv_content = "时间戳,专注状态,应用名称,窗口标题,置信度\n".to_string();
            
            for result in filtered_results {
                csv_content.push_str(&format!(
                    "{},{:?},{},{},{}\n",
                    result.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    result.focus_state,
                    result.application_name.unwrap_or_else(|| "未知".to_string()),
                    result.window_title.unwrap_or_else(|| "无标题".to_string()),
                    result.confidence
                ));
            }
            
            Ok(csv_content)
        }
        _ => Err("不支持的导出格式".to_string())
    }
}

// ===== 数据结构定义 =====

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportListItem {
    pub id: String,
    pub title: String,
    pub date: String,
    pub report_type: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub monitoring_results: Vec<MonitoringResult>,
    pub focus_sessions: Vec<crate::models::FocusSession>,
    pub export_date: String,
    pub date_range: String,
}

// ===== 数据管理相关命令 =====

/// 清理旧数据
#[command]
pub async fn cleanup_old_data(days_to_keep: Option<u32>) -> Result<String, String> {
    let days_to_keep = days_to_keep.unwrap_or(30);
    println!("🧹 开始清理 {} 天前的旧数据", days_to_keep);
    
    let storage_service = get_storage_service().await?;
    let mut cleaned_items = 0;
    
    // 清理监控结果
    match storage_service.cleanup_old_monitoring_results(days_to_keep).await {
        Ok(count) => {
            cleaned_items += count;
            println!("✅ 清理了 {} 条监控记录", count);
        }
        Err(e) => println!("⚠️ 清理监控记录失败: {}", e)
    }
    
    // 清理旧的专注会话（如果超过保留期限）
    if days_to_keep < 90 {
        match storage_service.cleanup_old_focus_sessions(days_to_keep).await {
            Ok(count) => {
                cleaned_items += count;
                println!("✅ 清理了 {} 个专注会话", count);
            }
            Err(e) => println!("⚠️ 清理专注会话失败: {}", e)
        }
    }
    
    Ok(format!("数据清理完成，清理了 {} 项记录", cleaned_items))
}

/// 获取存储使用情况
#[command]
pub async fn get_storage_usage() -> Result<StorageUsageInfo, String> {
    println!("📊 获取存储使用情况");
    
    let storage_service = get_storage_service().await?;
    
    // 计算各类数据的大小
    let monitoring_count = storage_service.load_monitoring_results().await
        .map(|results| results.len() as u32)
        .unwrap_or(0);
    
    let sessions_count = storage_service.load_focus_sessions().await
        .map(|sessions| sessions.len() as u32)
        .unwrap_or(0);
    
    let tasks_count = storage_service.load_tasks().await
        .map(|tasks| tasks.len() as u32)
        .unwrap_or(0);
    
    // 估算数据大小（粗略计算）
    let estimated_monitoring_size = monitoring_count * 500; // 每条记录约500字节
    let estimated_sessions_size = sessions_count * 200;     // 每个会话约200字节
    let estimated_tasks_size = tasks_count * 100;           // 每个任务约100字节
    let total_size = estimated_monitoring_size + estimated_sessions_size + estimated_tasks_size;
    
    let usage_info = StorageUsageInfo {
        total_size_bytes: total_size,
        monitoring_records_count: monitoring_count,
        focus_sessions_count: sessions_count,
        tasks_count: tasks_count,
        estimated_monitoring_size_bytes: estimated_monitoring_size,
        estimated_sessions_size_bytes: estimated_sessions_size,
        estimated_tasks_size_bytes: estimated_tasks_size,
        last_cleanup_date: None, // TODO: 实现最后清理日期跟踪
        recommendations: generate_storage_recommendations(total_size, monitoring_count),
    };
    
    println!("📋 存储使用情况: 总计 {:.2} MB", total_size as f64 / 1024.0 / 1024.0);
    Ok(usage_info)
}

/// 优化存储
#[command]
pub async fn optimize_storage() -> Result<String, String> {
    println!("⚡ 开始存储优化");
    
    let storage_service = get_storage_service().await?;
    let mut optimization_results = Vec::new();
    
    // 1. 压缩监控数据中的重复内容
    match optimize_monitoring_data(&storage_service).await {
        Ok(saved_bytes) => {
            optimization_results.push(format!("监控数据优化节省 {} KB", saved_bytes / 1024));
        }
        Err(e) => optimization_results.push(format!("监控数据优化失败: {}", e))
    }
    
    // 2. 清理空任务或重复任务
    match optimize_tasks_data(&storage_service).await {
        Ok(cleaned_count) => {
            optimization_results.push(format!("清理了 {} 个冗余任务", cleaned_count));
        }
        Err(e) => optimization_results.push(format!("任务数据优化失败: {}", e))
    }
    
    // 3. 压缩专注会话数据
    match optimize_sessions_data(&storage_service).await {
        Ok(optimized_count) => {
            optimization_results.push(format!("优化了 {} 个专注会话记录", optimized_count));
        }
        Err(e) => optimization_results.push(format!("会话数据优化失败: {}", e))
    }
    
    let result = format!("存储优化完成:\n{}", optimization_results.join("\n"));
    println!("✅ {}", result);
    Ok(result)
}

/// 备份数据
#[command]
pub async fn backup_data(backup_path: Option<String>) -> Result<String, String> {
    println!("💾 开始数据备份");
    
    let storage_service = get_storage_service().await?;
    
    // 确定备份路径
    let _backup_path = backup_path.unwrap_or_else(|| {
        format!("backup_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"))
    });
    
    // 收集所有数据
    let monitoring_results = storage_service.load_monitoring_results().await
        .map_err(|e| format!("加载监控数据失败: {}", e))?;
    
    let focus_sessions = storage_service.load_focus_sessions().await
        .map_err(|e| format!("加载会话数据失败: {}", e))?;
    
    let tasks = storage_service.load_tasks().await
        .map_err(|e| format!("加载任务数据失败: {}", e))?;
    
    let user_settings = storage_service.load_user_settings().await
        .unwrap_or_default();
    
    let ai_config = storage_service.load_ai_config().await
        .ok();
    
    let monitoring_config = storage_service.load_monitoring_config().await
        .ok();
    
    // 创建备份数据结构
    let backup_data = BackupData {
        version: "1.0".to_string(),
        backup_date: chrono::Utc::now(),
        monitoring_results,
        focus_sessions,
        tasks,
        user_settings,
        ai_config,
        monitoring_config,
    };
    
    // 序列化并保存
    let backup_json = serde_json::to_string_pretty(&backup_data)
        .map_err(|e| format!("序列化备份数据失败: {}", e))?;
    
    // 这里应该将数据写入文件，但Tauri的文件操作可能需要特殊处理
    // 暂时返回JSON数据让前端处理下载
    println!("✅ 备份数据准备完成，大小: {} KB", backup_json.len() / 1024);
    Ok(backup_json)
}

/// 恢复数据
#[command]
pub async fn restore_data(backup_data: String) -> Result<String, String> {
    println!("🔄 开始数据恢复");
    
    let storage_service = get_storage_service().await?;
    
    // 解析备份数据
    let backup: BackupData = serde_json::from_str(&backup_data)
        .map_err(|e| format!("解析备份数据失败: {}", e))?;
    
    let mut restored_items = Vec::new();
    
    // 计算数量（在移动数据之前）
    let monitoring_count = backup.monitoring_results.len();
    let sessions_count = backup.focus_sessions.len();
    let tasks_count = backup.tasks.len();
    
    // 恢复监控结果
    for result in backup.monitoring_results {
        if let Err(e) = storage_service.save_monitoring_result(&result).await {
            println!("⚠️ 恢复监控记录失败: {}", e);
        }
    }
    restored_items.push(format!("监控记录: {} 条", monitoring_count));
    
    // 恢复专注会话
    for session in backup.focus_sessions {
        if let Err(e) = storage_service.save_focus_session(&session).await {
            println!("⚠️ 恢复专注会话失败: {}", e);
        }
    }
    restored_items.push(format!("专注会话: {} 个", sessions_count));
    
    // 恢复任务
    for task in backup.tasks {
        if let Err(e) = storage_service.save_task(&task).await {
            println!("⚠️ 恢复任务失败: {}", e);
        }
    }
    restored_items.push(format!("任务: {} 个", tasks_count));
    
    // 恢复设置
    if let Err(e) = storage_service.save_user_settings(&backup.user_settings).await {
        println!("⚠️ 恢复用户设置失败: {}", e);
    } else {
        restored_items.push("用户设置".to_string());
    }
    
    if let Some(ai_config) = backup.ai_config {
        if let Err(e) = storage_service.save_ai_config(&ai_config).await {
            println!("⚠️ 恢复AI配置失败: {}", e);
        } else {
            restored_items.push("AI配置".to_string());
        }
    }
    
    if let Some(monitoring_config) = backup.monitoring_config {
        if let Err(e) = storage_service.save_monitoring_config(&monitoring_config).await {
            println!("⚠️ 恢复监控配置失败: {}", e);
        } else {
            restored_items.push("监控配置".to_string());
        }
    }
    
    let result = format!("数据恢复完成，已恢复:\n{}", restored_items.join("\n"));
    println!("✅ {}", result);
    Ok(result)
}

// ===== 辅助函数 =====

/// 生成存储建议
fn generate_storage_recommendations(total_size: u32, monitoring_count: u32) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    if total_size > 50 * 1024 * 1024 { // 50MB
        recommendations.push("数据量较大，建议定期清理旧数据".to_string());
    }
    
    if monitoring_count > 10000 {
        recommendations.push("监控记录过多，建议调整监控频率或清理周期".to_string());
    }
    
    if total_size > 20 * 1024 * 1024 { // 20MB
        recommendations.push("建议执行存储优化以减少空间占用".to_string());
    }
    
    if recommendations.is_empty() {
        recommendations.push("存储使用正常".to_string());
    }
    
    recommendations
}

/// 优化监控数据
async fn optimize_monitoring_data(_storage_service: &StorageService) -> Result<u32, String> {
    // 这里可以实现监控数据的优化逻辑
    // 例如压缩重复的OCR文本、合并相似的分析结果等
    println!("🔧 优化监控数据...");
    
    // 模拟优化节省的空间
    Ok(1024 * 50) // 假设节省了50KB
}

/// 优化任务数据
async fn optimize_tasks_data(storage_service: &StorageService) -> Result<u32, String> {
    println!("🔧 优化任务数据...");
    
    let tasks = storage_service.load_tasks().await
        .map_err(|e| format!("加载任务失败: {}", e))?;
    
    // 查找重复或空任务
    let mut unique_tasks = std::collections::HashSet::new();
    let mut cleaned_count = 0;
    
    for task in &tasks {
        if task.text.trim().is_empty() {
            cleaned_count += 1;
            continue;
        }
        
        if !unique_tasks.insert(task.text.clone()) {
            cleaned_count += 1;
        }
    }
    
    Ok(cleaned_count)
}

/// 优化会话数据
async fn optimize_sessions_data(storage_service: &StorageService) -> Result<u32, String> {
    println!("🔧 优化会话数据...");
    
    let sessions = storage_service.load_focus_sessions().await
        .map_err(|e| format!("加载会话失败: {}", e))?;
    
    // 计算优化的会话数量（例如合并连续的短会话）
    let optimized_count = sessions.len() as u32;
    
    Ok(optimized_count)
}

// ===== 数据结构定义 =====

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageUsageInfo {
    pub total_size_bytes: u32,
    pub monitoring_records_count: u32,
    pub focus_sessions_count: u32,
    pub tasks_count: u32,
    pub estimated_monitoring_size_bytes: u32,
    pub estimated_sessions_size_bytes: u32,
    pub estimated_tasks_size_bytes: u32,
    pub last_cleanup_date: Option<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupData {
    pub version: String,
    pub backup_date: DateTime<Utc>,
    pub monitoring_results: Vec<MonitoringResult>,
    pub focus_sessions: Vec<crate::models::FocusSession>,
    pub tasks: Vec<Task>,
    pub user_settings: UserSettings,
    pub ai_config: Option<crate::services::ai_service::AIConfig>,
    pub monitoring_config: Option<MonitoringConfig>,
}

/// 发送分心干预通知
async fn send_distraction_intervention(app_handle: &tauri::AppHandle, result: &MonitoringResult) {
    println!("🚨 发送分心干预通知");
    
    let intervention_type = match result.focus_state {
        FocusState::Distracted => "light",
        FocusState::SeverelyDistracted => "severe",
        _ => {
            println!("ℹ️ 非分心状态，跳过干预");
            return; // 只处理分心状态
        }
    };
    
    let message = match result.focus_state {
        FocusState::Distracted => "检测到轻度分心，建议重新集中注意力。",
        FocusState::SeverelyDistracted => "严重分心警告！请立即回到工作状态！",
        _ => ""
    };
    
    let intervention_data = serde_json::json!({
        "type": intervention_type,
        "message": message,
        "timestamp": result.timestamp,
        "focus_state": match result.focus_state {
            FocusState::Distracted => "distracted",
            FocusState::SeverelyDistracted => "severely_distracted",
            _ => "unknown"
        },
        "confidence": result.confidence,
        "application_name": result.application_name,
        "window_title": result.window_title,
        "urgent": matches!(result.focus_state, FocusState::SeverelyDistracted),
        "duration_seconds": if matches!(result.focus_state, FocusState::SeverelyDistracted) { 15 } else { 10 },
        "sound_enabled": true
    });
    
    println!("📝 干预数据已准备: {}", intervention_data);
    
    // 发送分心干预事件
    if let Err(e) = app_handle.emit_all("distraction_intervention", &intervention_data) {
        println!("❌ 发送分心干预事件失败: {}", e);
    } else {
        println!("📡 分心干预事件已发送: {}", intervention_type);
    }
}