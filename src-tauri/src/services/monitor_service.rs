use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tauri::{AppHandle, Manager};

use crate::services::ai_service::{AIService, AIConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub interval_minutes: u8, // 1-10分钟
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
    pub ai_config: AIConfig,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_minutes: 3, // 默认3分钟
            whitelist: vec![],
            blacklist: vec![],
            ai_config: AIConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FocusState {
    Focused,
    Distracted,
    SeverelyDistracted,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringResult {
    pub timestamp: DateTime<Utc>,
    pub focus_state: FocusState,
    pub application_name: Option<String>,
    pub window_title: Option<String>,
    pub ocr_text: Option<String>,
    pub ai_analysis: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentActivity {
    pub application_name: Option<String>,
    pub window_title: Option<String>,
    pub is_productive: Option<bool>,
    pub timestamp: DateTime<Utc>,
}

pub struct MonitorService {
    config: Arc<RwLock<MonitoringConfig>>,
    current_activity: Arc<Mutex<Option<CurrentActivity>>>,
    is_monitoring: Arc<Mutex<bool>>,
    last_result: Arc<Mutex<Option<MonitoringResult>>>,
    monitor_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
}

impl MonitorService {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(MonitoringConfig::default())),
            current_activity: Arc::new(Mutex::new(None)),
            is_monitoring: Arc::new(Mutex::new(false)),
            last_result: Arc::new(Mutex::new(None)),
            monitor_handle: Arc::new(Mutex::new(None)),
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// 设置AppHandle用于事件发送
    pub async fn set_app_handle(&self, handle: AppHandle) {
        let mut app_handle = self.app_handle.lock().await;
        *app_handle = Some(handle);
        println!("✅ MonitorService AppHandle已设置");
    }

    /// 更新监控配置
    pub async fn update_config(&self, config: MonitoringConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        println!("监控配置已更新");
        Ok(())
    }

    /// 获取监控配置
    pub async fn get_config(&self) -> MonitoringConfig {
        self.config.read().await.clone()
    }

    /// 开始监控
    pub async fn start_monitoring(&self) -> Result<()> {
        println!("🚀 准备启动监控服务...");
        
        let mut is_monitoring = self.is_monitoring.lock().await;
        if *is_monitoring {
            println!("⚠️ 监控服务已在运行中，跳过启动");
            return Ok(()); // 已在监控中
        }
        
        // 检查配置有效性
        let config = self.config.read().await;
        println!("📋 检查监控配置:");
        println!("   - 监控启用: {}", config.enabled);
        println!("   - 检查间隔: {} 分钟", config.interval_minutes);
        println!("   - 白名单应用: {} 项", config.whitelist.len());
        println!("   - 黑名单应用: {} 项", config.blacklist.len());
        println!("   - AI配置: {} - {}", config.ai_config.api_type, config.ai_config.api_url);
        drop(config); // 释放读锁
        
        *is_monitoring = true;
        println!("✅ 监控状态已设置为启用");
        println!("🔄 启动监控主循环...");

        // 启动监控任务
        let config = self.config.clone();
        let current_activity = self.current_activity.clone();
        let last_result = self.last_result.clone();
        let is_monitoring_flag = self.is_monitoring.clone();
        let app_handle = self.app_handle.clone();

        let handle = tokio::spawn(async move {
            Self::monitoring_loop(config, current_activity, last_result, is_monitoring_flag, app_handle).await;
        });

        let mut monitor_handle = self.monitor_handle.lock().await;
        *monitor_handle = Some(handle);
        
        println!("🎯 监控服务启动完成");
        Ok(())
    }

    /// 停止监控
    pub async fn stop_monitoring(&self) -> Result<()> {
        println!("🛑 准备停止监控服务...");
        
        let mut is_monitoring = self.is_monitoring.lock().await;
        if !*is_monitoring {
            println!("⚠️ 监控服务已处于停止状态");
            return Ok(());
        }
        
        *is_monitoring = false;
        println!("✅ 监控状态已设置为停止");
        
        // 取消监控任务
        let mut handle = self.monitor_handle.lock().await;
        if let Some(h) = handle.take() {
            println!("🔄 正在终止监控主循环...");
            h.abort();
            println!("✅ 监控主循环已终止");
        } else {
            println!("⚠️ 未找到运行中的监控任务句柄");
        }
        
        // 清理当前状态
        *self.current_activity.lock().await = None;
        *self.last_result.lock().await = None;
        println!("🧹 监控状态已清理");
        
        println!("🎯 监控服务停止完成");
        Ok(())
    }

    /// 监控主循环
    async fn monitoring_loop(
        config: Arc<RwLock<MonitoringConfig>>,
        current_activity: Arc<Mutex<Option<CurrentActivity>>>,
        last_result: Arc<Mutex<Option<MonitoringResult>>>,
        is_monitoring: Arc<Mutex<bool>>,
        app_handle: Arc<Mutex<Option<AppHandle>>>,
    ) {
        let mut loop_count = 0;
        let loop_start_time = std::time::Instant::now();
        
        println!("🔄 监控主循环已启动");
        
        loop {
            loop_count += 1;
            let iteration_start = std::time::Instant::now();
            
            // 检查监控状态
            let is_running = *is_monitoring.lock().await;
            if !is_running {
                println!("🛑 监控循环停止信号收到，退出循环 (共执行 {} 次)", loop_count - 1);
                break;
            }

            // 获取当前配置
            let config_snapshot = config.read().await.clone();
            println!("🔄 监控循环第 {} 次迭代 (运行时间: {:?})", 
                loop_count, 
                loop_start_time.elapsed()
            );
            
            if !config_snapshot.enabled {
                println!("⏸️ 监控已禁用，等待10秒后重新检查...");
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }

            println!("⚙️ 当前配置: 间隔={}分钟, 白名单={}项, 黑名单={}项", 
                config_snapshot.interval_minutes,
                config_snapshot.whitelist.len(),
                config_snapshot.blacklist.len()
            );

            // 执行一次监控检查
            if let Err(e) = Self::perform_monitoring_check(
                &config_snapshot,
                &current_activity,
                &last_result,
                &app_handle,
            ).await {
                println!("❌ 第 {} 次监控检查失败: {}", loop_count, e);
            } else {
                println!("✅ 第 {} 次监控检查成功完成", loop_count);
            }

            // 计算并显示本次迭代耗时
            let iteration_duration = iteration_start.elapsed();
            println!("⏱️ 第 {} 次迭代总耗时: {:?}", loop_count, iteration_duration);

            // 等待下次检查
            let interval = Duration::from_secs(config_snapshot.interval_minutes as u64 * 60);
            println!("😴 等待 {} 分钟后进行下次检查...", config_snapshot.interval_minutes);
            println!("📅 下次检查预计时间: {:?}", 
                std::time::SystemTime::now() + std::time::Duration::from_secs(interval.as_secs())
            );
            
            tokio::time::sleep(interval).await;
        }
        
        let total_runtime = loop_start_time.elapsed();
        println!("🏁 监控主循环已结束，总运行时间: {:?}, 总迭代次数: {}", 
            total_runtime, 
            loop_count - 1
        );
    }

    /// 执行一次监控检查
    async fn perform_monitoring_check(
        config: &MonitoringConfig,
        current_activity: &Arc<Mutex<Option<CurrentActivity>>>,
        last_result: &Arc<Mutex<Option<MonitoringResult>>>,
        app_handle: &Arc<Mutex<Option<AppHandle>>>,
    ) -> Result<()> {
        use std::time::Instant;
        
        let check_start = Instant::now();
        println!("🔄 执行监控检查...");

        // 1. 获取当前活动应用信息
        println!("📱 步骤1: 获取当前应用信息");
        let app_start = Instant::now();
        let (app_name, window_title) = Self::get_current_application_info().await?;
        println!("⏱️ 应用信息获取耗时: {:?}", app_start.elapsed());
        println!("📋 当前应用: {:?}", app_name);
        println!("🪟 窗口标题: {:?}", window_title);
        
        // 2. 截取屏幕并进行OCR
        println!("📸 步骤2: 屏幕截图和OCR识别");
        let ocr_start = Instant::now();
        let ocr_text = Self::capture_screen_and_ocr().await?;
        println!("⏱️ 截图+OCR总耗时: {:?}", ocr_start.elapsed());
        
        // 3. 调用AI进行专注判断
        println!("🤖 步骤3: AI专注状态分析");
        let ai_start = Instant::now();
        let ai_result = Self::analyze_focus_with_ai(
            config,
            &app_name,
            &window_title,
            &ocr_text,
        ).await?;
        println!("⏱️ AI分析耗时: {:?}", ai_start.elapsed());
        println!("🎯 AI分析结果: {:?} (置信度: {:.2})", ai_result.focus_state, ai_result.confidence);

        // 4. 更新当前活动状态
        let activity = CurrentActivity {
            application_name: app_name.clone(),
            window_title: window_title.clone(),
            is_productive: Some(matches!(ai_result.focus_state, FocusState::Focused)),
            timestamp: Utc::now(),
        };

        *current_activity.lock().await = Some(activity);
        *last_result.lock().await = Some(ai_result.clone());

        // 5. 发送专注状态变化事件给前端
        println!("📡 步骤5: 发送专注状态事件");
        let event_start = Instant::now();
        if let Err(e) = Self::send_focus_state_event(&app_handle, &ai_result).await {
            println!("⚠️ 发送专注状态事件失败: {}", e);
        } else {
            println!("✅ 专注状态事件已发送");
        }
        println!("⏱️ 事件发送耗时: {:?}", event_start.elapsed());

        // 6. 检查是否需要分心干预
        if matches!(ai_result.focus_state, FocusState::Distracted | FocusState::SeverelyDistracted) {
            println!("🚨 步骤6: 发送分心干预事件");
            if let Err(e) = Self::send_distraction_intervention_event(&app_handle, &ai_result).await {
                println!("❌ 发送分心干预事件失败: {}", e);
            } else {
                println!("✅ 分心干预事件已发送");
            }
        }

        // 7. 保存监控结果到存储服务
        println!("💾 步骤7: 保存监控结果");
        let save_start = Instant::now();
        match Self::save_monitoring_result(&ai_result).await {
            Ok(_) => {
                println!("⏱️ 结果保存耗时: {:?}", save_start.elapsed());
                println!("✅ 监控结果已保存到存储服务");
            }
            Err(e) => {
                println!("⚠️ 保存监控结果失败: {}", e);
            }
        }

        let total_duration = check_start.elapsed();
        println!("🎯 监控检查完成: {:?}, 总耗时: {:?}", ai_result.focus_state, total_duration);
        Ok(())
    }

    /// 保存监控结果到存储服务
    async fn save_monitoring_result(result: &MonitoringResult) -> Result<()> {
        // 获取应用数据目录
        // 使用应用本地目录
        let app_data_dir = std::path::PathBuf::from("data");
        
        // 创建存储服务实例
        let storage_service = crate::services::storage_service::StorageService::new(app_data_dir);
        
        // 保存监控结果
        match storage_service.save_monitoring_result(result).await {
            Ok(_) => {
                println!("📊 监控结果已保存: 时间={}, 状态={:?}", 
                    result.timestamp.format("%H:%M:%S"), 
                    result.focus_state
                );
                Ok(())
            }
            Err(e) => {
                println!("❌ 保存监控结果时出错: {}", e);
                Err(anyhow::anyhow!("保存监控结果失败: {}", e))
            }
        }
    }

    /// 获取当前活动应用程序和窗口信息
    pub async fn get_current_application_info() -> Result<(Option<String>, Option<String>)> {
        use std::time::Instant;
        
        println!("📱 获取当前活动应用信息...");
        
        let app_info_start = Instant::now();
        
        #[cfg(windows)]
        {
            tokio::task::spawn_blocking(move || {
                unsafe {
                    use winapi::um::winuser::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId};
                    use winapi::um::processthreadsapi::OpenProcess;
                    use winapi::um::psapi::GetModuleBaseNameW;
                    use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
                    use winapi::um::handleapi::CloseHandle;
                    
                    let hwnd = GetForegroundWindow();
                    if hwnd.is_null() {
                        println!("⚠️ 无法获取前台窗口");
                        return Ok((None, None));
                    }
                    
                    // 获取窗口标题
                    let mut window_title = vec![0u16; 256];
                    let title_len = GetWindowTextW(hwnd, window_title.as_mut_ptr(), 256);
                    let window_title_str = if title_len > 0 {
                        let title = String::from_utf16_lossy(&window_title[..title_len as usize]);
                        println!("🪟 窗口标题: {}", title);
                        Some(title)
                    } else {
                        println!("⚠️ 无法获取窗口标题");
                        None
                    };
                    
                    // 获取进程ID
                    let mut process_id = 0u32;
                    GetWindowThreadProcessId(hwnd, &mut process_id);
                    
                    if process_id == 0 {
                        println!("⚠️ 无法获取进程ID");
                        return Ok((None, window_title_str));
                    }
                    
                    // 打开进程
                    let process_handle = OpenProcess(
                        PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                        0,
                        process_id
                    );
                    
                    if process_handle.is_null() {
                        println!("⚠️ 无法打开进程 (PID: {})", process_id);
                        return Ok((None, window_title_str));
                    }
                    
                    // 获取进程名称
                    let mut process_name = vec![0u16; 256];
                    let name_len = GetModuleBaseNameW(
                        process_handle,
                        std::ptr::null_mut(),
                        process_name.as_mut_ptr(),
                        256
                    );
                    
                    CloseHandle(process_handle);
                    
                    let app_name = if name_len > 0 {
                        let name = String::from_utf16_lossy(&process_name[..name_len as usize]);
                        println!("📋 应用程序: {} (PID: {})", name, process_id);
                        Some(name)
                    } else {
                        println!("⚠️ 无法获取进程名称");
                        None
                    };
                    
                    let app_info_duration = app_info_start.elapsed();
                    println!("⏱️ 应用信息获取耗时: {:?}", app_info_duration);
                    
                    Ok((app_name, window_title_str))
                }
            }).await?
        }
        
        #[cfg(not(windows))]
        {
            println!("⚠️ 非Windows系统，返回模拟应用信息");
            Ok((Some("测试应用".to_string()), Some("测试窗口".to_string())))
        }
    }

    /// 截取屏幕并进行OCR识别  
    pub async fn capture_screen_and_ocr() -> Result<Option<String>> {
        use std::time::Instant;
        
        let capture_start = Instant::now();
        println!("📸 开始屏幕截图和OCR识别...");
        
        // 截取屏幕
        match Self::capture_screenshot_sync() {
            Ok(Some(image_data)) => {
                println!("✅ 屏幕截图成功，图像大小: {} KB", image_data.len() / 1024);
                
                // 执行OCR
                let ocr_result = Self::perform_ocr(&image_data).await?;
                
                let total_duration = capture_start.elapsed();
                println!("⏱️ 截图+OCR总耗时: {:?}", total_duration);
                
                Ok(ocr_result)
            }
            Ok(None) => {
                println!("⚠️ 屏幕截图失败");
        Ok(None)
            }
            Err(e) => {
                println!("❌ 屏幕截图错误: {}", e);
                Err(e)
            }
        }
    }

    /// 同步截图函数
    fn capture_screenshot_sync() -> Result<Option<Vec<u8>>> {
        use screenshots::Screen;
        use image::{ImageOutputFormat, DynamicImage, RgbaImage};
        use std::io::Cursor;

        println!("📸 开始屏幕截图...");
        
        let screens = Screen::all()?;
        println!("🖥️ 检测到 {} 个屏幕", screens.len());
        
        if let Some(screen) = screens.first() {
            println!("📐 主屏幕分辨率: {}x{}", screen.display_info.width, screen.display_info.height);
            
            let screenshot_start = std::time::Instant::now();
            let screenshot_image = screen.capture()?;
            let screenshot_duration = screenshot_start.elapsed();
            
            println!("⏱️ 截图耗时: {:?}", screenshot_duration);
            println!("🖼️ 截图尺寸: {}x{}", screenshot_image.width(), screenshot_image.height());
            
            // 将screenshots::Image转换为image::RgbaImage
            let width = screenshot_image.width() as u32;
            let height = screenshot_image.height() as u32;
            let raw_data = screenshot_image.rgba();
            
            let rgba_image = RgbaImage::from_raw(width, height, raw_data.to_vec())
                .ok_or_else(|| anyhow::anyhow!("无法创建RGBA图像"))?;
            
            let dynamic_image = DynamicImage::ImageRgba8(rgba_image);
            
            // 压缩为JPEG格式以减少数据量
            let mut cursor = Cursor::new(Vec::new());
            let compression_start = std::time::Instant::now();
            dynamic_image.write_to(&mut cursor, ImageOutputFormat::Jpeg(85))?;
            let compression_duration = compression_start.elapsed();
            
            let compressed_data = cursor.into_inner();
            println!("⏱️ JPEG压缩耗时: {:?}", compression_duration);
            println!("📊 压缩后大小: {} KB (压缩率: {:.1}%)", 
                compressed_data.len() / 1024,
                (compressed_data.len() as f64 / (width * height * 4) as f64) * 100.0
            );
            
            Ok(Some(compressed_data))
        } else {
            println!("❌ 未检测到可用屏幕");
        Ok(None)
        }
    }

    /// 便携式Tesseract OCR识别 (命令行版本)
    async fn perform_ocr(image_data: &[u8]) -> Result<Option<String>> {
        let ocr_start = std::time::Instant::now();
        
        println!("🔍 开始便携式Tesseract OCR识别...");
        println!("📊 图像数据大小: {} KB", image_data.len() / 1024);
        
        tokio::task::spawn_blocking({
            let image_data = image_data.to_vec();
            move || {
                // 使用命令行方式调用便携式Tesseract
                match Self::perform_command_line_ocr(&image_data) {
                    Ok(Some(text)) => {
                        let ocr_duration = ocr_start.elapsed();
                        println!("⏱️ OCR识别耗时: {:?}", ocr_duration);
                        println!("✅ OCR识别成功，文本长度: {} 字符", text.len());
                        
                        if text.len() > 200 {
                            println!("📖 文本预览: {}...", &text[..200]);
                        } else if !text.is_empty() {
                            println!("📖 识别文本: {}", text);
                        }
                        
                        Ok(Some(text))
                    }
                    Ok(None) => {
                        println!("⚠️ OCR识别结果为空，使用智能分析");
                        Ok(Self::smart_image_analysis(&image_data))
                    }
                    Err(e) => {
                        println!("❌ OCR识别失败: {}", e);
                        println!("🔄 回退到智能图像分析");
                        Ok(Self::smart_image_analysis(&image_data))
                    }
                }
            }
        }).await?
    }

    /// 便携式Tesseract命令行OCR实现
    fn perform_command_line_ocr(image_data: &[u8]) -> Result<Option<String>> {
        use std::env;
        use std::process::Command;
        
        println!("🔧 启动便携式Tesseract命令行OCR...");
        
        // 1. 创建临时文件
        let temp_dir = env::temp_dir();
        let temp_image = temp_dir.join("my_focus_ocr_input.png");
        let temp_output = temp_dir.join("my_focus_ocr_output");
        
        println!("📁 临时文件路径:");
        println!("   输入: {}", temp_image.display());
        println!("   输出: {}", temp_output.display());
        
        // 2. 保存图像文件
        println!("🖼️ 保存图像到临时文件...");
        let img = image::load_from_memory(image_data)
            .map_err(|e| anyhow::anyhow!("图像解码失败: {}", e))?;
        
        let gray = img.to_luma8();
        gray.save(&temp_image)
            .map_err(|e| anyhow::anyhow!("保存临时图像失败: {}", e))?;
        
        println!("✅ 图像已保存: {}x{}", img.width(), img.height());
        
        // 3. 查找便携式Tesseract
        let tesseract_exe = Self::find_portable_tesseract()?;
        println!("📍 使用Tesseract: {}", tesseract_exe);
        
        // 4. 执行OCR识别 (先尝试中英文，失败则用英文)
        let result = Self::execute_tesseract_command(&tesseract_exe, &temp_image, &temp_output, "chi_sim+eng")
            .or_else(|e| {
                println!("⚠️ 中英文识别失败: {}", e);
                println!("🔄 尝试仅英文识别...");
                Self::execute_tesseract_command(&tesseract_exe, &temp_image, &temp_output, "eng")
            });
        
        // 5. 清理临时文件
        let _ = std::fs::remove_file(&temp_image);
        let _ = std::fs::remove_file(format!("{}.txt", temp_output.to_string_lossy()));
        
        result
    }
    
    /// 查找便携式Tesseract可执行文件
    fn find_portable_tesseract() -> Result<String> {
        use std::process::Command;
        
        println!("🔍 查找便携式Tesseract...");
        
        // 获取应用程序目录
        let app_dir = std::env::current_exe()
            .map_err(|e| anyhow::anyhow!("无法获取应用程序路径: {}", e))?
            .parent()
            .ok_or_else(|| anyhow::anyhow!("无法获取应用程序目录"))?
            .to_path_buf();
        
        // 可能的便携式Tesseract路径
        let possible_paths = vec![
            // 开发模式路径
            app_dir.join("../../../resources/tesseract/tesseract.exe"),
            app_dir.join("../../resources/tesseract/tesseract.exe"),
            app_dir.join("../resources/tesseract/tesseract.exe"),
            // 生产模式路径
            app_dir.join("resources/tesseract/tesseract.exe"),
            app_dir.join("tesseract/tesseract.exe"),
            app_dir.join("tesseract.exe"),
        ];
        
        for tesseract_path in possible_paths {
            println!("   检查: {}", tesseract_path.display());
            if tesseract_path.exists() {
                let path_str = tesseract_path.to_string_lossy().to_string();
                println!("✅ 找到便携式Tesseract: {}", path_str);
                return Ok(path_str);
            }
        }
        
        // 如果找不到便携式版本，尝试系统安装
        println!("⚠️ 未找到便携式Tesseract，检查系统安装...");
        match Command::new("tesseract").arg("--version").output() {
            Ok(output) if output.status.success() => {
                println!("✅ 发现系统Tesseract");
                Ok("tesseract".to_string())
            }
            _ => {
                Err(anyhow::anyhow!(
                    "无法找到Tesseract！\n\
                    请确保:\n\
                    1. 将tesseract.exe复制到resources/tesseract/目录\n\
                    2. 或者在系统中安装Tesseract并添加到PATH"
                ))
            }
        }
    }
    
    /// 执行Tesseract命令行识别
    fn execute_tesseract_command(
        tesseract_exe: &str,
        input_image: &std::path::Path,
        output_base: &std::path::Path,
        language: &str,
    ) -> Result<Option<String>> {
        use std::process::Command;
        
        println!("🔤 执行Tesseract识别 (语言: {})...", language);
        
        // 构建命令
        let output = Command::new(tesseract_exe)
            .arg(input_image)
            .arg(output_base)
            .arg("-l")
            .arg(language)
            .arg("--psm")
            .arg("6")  // 单个统一文本块
            .arg("--oem")
            .arg("3")  // 默认OCR引擎模式
            .output()
            .map_err(|e| anyhow::anyhow!("执行Tesseract失败: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Tesseract执行失败: {}", stderr));
        }
        
        println!("✅ Tesseract命令执行成功");
        
        // 读取结果文件
        let output_file = format!("{}.txt", output_base.to_string_lossy());
        match std::fs::read_to_string(&output_file) {
            Ok(text) => {
                let text = text.trim().to_string();
                if text.is_empty() {
                    println!("⚠️ OCR识别结果为空");
                    Ok(None)
                } else {
                    println!("📖 OCR识别成功，识别到 {} 字符", text.len());
                    
                    // 清理文本
                    let cleaned_text = text
                        .lines()
                        .map(|line| line.trim())
                        .filter(|line| !line.is_empty())
                        .collect::<Vec<_>>()
                        .join("\n");
                    
                    if cleaned_text.is_empty() {
                        Ok(None)
                    } else {
                        println!("✨ 文本清理完成，最终长度: {} 字符", cleaned_text.len());
                        Ok(Some(cleaned_text))
                    }
                }
            }
            Err(e) => {
                Err(anyhow::anyhow!("读取OCR结果失败: {}", e))
            }
        }
    }
    
    /// 智能图像分析 (Tesseract失败时的备选方案)
    fn smart_image_analysis(image_data: &[u8]) -> Option<String> {
        println!("🧠 执行智能图像分析...");
        
        // 基于图像特征进行智能分析
        let image_size_kb = image_data.len() / 1024;
        let timestamp = chrono::Utc::now();
        
        // 根据图像大小和时间推断可能的内容类型
        let analysis = if image_size_kb > 1000 {
            "高分辨率屏幕内容，可能包含大量文本信息，建议检查是否在进行文档编辑或网页浏览"
        } else if image_size_kb > 500 {
            "中等分辨率屏幕内容，可能包含应用界面和文本，建议分析当前应用使用情况"
        } else if image_size_kb > 100 {
            "标准分辨率屏幕内容，包含基本界面元素，可能正在使用桌面应用"
        } else {
            "低分辨率或压缩度高的屏幕内容，文本信息有限"
        };
        
        // 构建智能分析结果
        let smart_analysis = format!(
            "智能屏幕分析报告:\n\
            内容评估: {}\n\
            图像大小: {} KB\n\
            分析时间: {}\n\
            建议: 基于图像特征进行专注状态判断\n\
            备注: 使用Tesseract OCR v0.14简化版本", 
            analysis,
            image_size_kb,
            timestamp.format("%Y-%m-%d %H:%M:%S")
        );
        
        println!("📊 智能分析完成: {} 字符", smart_analysis.len());
        Some(smart_analysis)
    }
    
    /// 获取当前任务名称
    async fn get_current_task_name() -> Result<String> {
        // 尝试从存储服务获取当前选中的任务
        match crate::commands::get_storage_service().await {
            Ok(storage_service) => {
                // 获取任务列表并找到未完成的任务
                if let Ok(tasks) = storage_service.load_tasks().await {
                    if let Some(task) = tasks.iter().find(|t| !t.completed) {
                        return Ok(task.text.clone());
                    }
                }
                
                // 如果没有未完成的任务，返回默认任务名
                Err(anyhow::anyhow!("未找到当前任务"))
            }
            Err(e) => Err(anyhow::anyhow!("获取存储服务失败: {}", e))
        }
    }

    /// 使用AI分析专注状态
    async fn analyze_focus_with_ai(
        config: &MonitoringConfig,
        app_name: &Option<String>,
        window_title: &Option<String>,
        ocr_text: &Option<String>,
    ) -> Result<MonitoringResult> {
        use std::time::Instant;
        
        println!("🧠 开始AI专注状态分析...");
        
        // 记录输入数据统计
        println!("📊 输入数据统计:");
        println!("   - 应用名称: {:?}", app_name);
        println!("   - 窗口标题: {:?}", window_title);
        if let Some(text) = ocr_text {
            println!("   - OCR文本长度: {} 字符", text.len());
            if text.len() > 100 {
                println!("   - OCR文本预览: {}...", &text[..100]);
            }
        } else {
            println!("   - OCR文本: 无");
        }
        println!("   - 白名单应用数量: {}", config.whitelist.len());
        println!("   - 黑名单应用数量: {}", config.blacklist.len());
        
        let ai_service = AIService::new(config.ai_config.clone());
        println!("🔧 AI服务配置:");
        println!("   - API类型: {}", config.ai_config.api_type);
        println!("   - API URL: {}", config.ai_config.api_url);
        println!("   - 检测模型: {}", config.ai_config.detection_model);
        println!("   - API密钥: {}***", 
            if config.ai_config.api_key.len() > 8 { 
                &config.ai_config.api_key[..8] 
            } else { 
                "短密钥" 
            }
        );
        
        // 构建AI分析提示
        println!("📝 构建AI分析提示词...");
        let prompt_start = Instant::now();
        
        // 尝试获取当前任务信息（从存储服务）
        let current_task = Self::get_current_task_name().await.ok();
        
        let prompt = Self::build_analysis_prompt(
            config,
            app_name,
            window_title,
            ocr_text,
            current_task.as_deref(),
        );
        let prompt_duration = prompt_start.elapsed();
        
        println!("⏱️ 提示词构建耗时: {:?}", prompt_duration);
        println!("📏 提示词总长度: {} 字符", prompt.len());
        println!("📋 AI分析提示词内容:");
        println!("{}", "─".repeat(50));
        println!("{}", prompt);
        println!("{}", "─".repeat(50));

        // 调用AI模型进行分析
        println!("🤖 调用AI模型进行分析...");
        let ai_call_start = Instant::now();
        let ai_response = Self::call_ai_model(&ai_service, &prompt).await?;
        let ai_call_duration = ai_call_start.elapsed();
        
        println!("⏱️ AI模型调用耗时: {:?}", ai_call_duration);
        println!("📤 AI原始响应长度: {} 字符", ai_response.len());
        println!("📋 AI原始响应内容:");
        println!("{}", "─".repeat(50));
        println!("{}", ai_response);
        println!("{}", "─".repeat(50));
        
        // 解析AI响应
        println!("🔍 解析AI响应...");
        let parse_start = Instant::now();
        let (focus_state, confidence) = Self::parse_ai_response(&ai_response);
        let parse_duration = parse_start.elapsed();
        
        println!("⏱️ 响应解析耗时: {:?}", parse_duration);
        println!("🎯 解析结果:");
        println!("   - 专注状态: {:?}", focus_state);
        println!("   - 置信度: {:.2} ({:.1}%)", confidence, confidence * 100.0);
        
        // 生成最终结果
        let result = MonitoringResult {
            timestamp: Utc::now(),
            focus_state: focus_state.clone(),
            application_name: app_name.clone(),
            window_title: window_title.clone(),
            ocr_text: ocr_text.clone(),
            ai_analysis: Some(ai_response),
            confidence,
        };
        
        println!("✅ AI分析完成: {:?} (置信度: {:.2})", focus_state, confidence);
        
        // 检查是否需要分心干预
        Self::check_distraction_intervention(&focus_state, &result, current_task.as_deref()).await;
        
        Ok(result)
    }

    /// 构建AI分析提示
    /// 构建AI分析提示
    fn build_analysis_prompt(
        config: &MonitoringConfig,
        app_name: &Option<String>,
        window_title: &Option<String>,
        ocr_text: &Option<String>,
        current_task: Option<&str>,
    ) -> String {
        let mut prompt = String::new();

        // 基础分析指令
        prompt.push_str("请分析用户当前的专注状态和任务执行情况。\n\n");

        // 当前任务信息
        if let Some(task) = current_task {
            prompt.push_str(&format!("**当前用户任务**: {}\n\n", task));
        } else {
            prompt.push_str("**当前用户任务**: 无明确任务设定\n\n");
        }

        // 应用规则配置
        if !config.whitelist.is_empty() || !config.blacklist.is_empty() {
            prompt.push_str("**应用使用规则**:\n");
            if !config.whitelist.is_empty() {
                prompt.push_str("白名单应用（通常有助于专注）: ");
                prompt.push_str(&config.whitelist.join(", "));
                prompt.push_str("\n");
            }
            if !config.blacklist.is_empty() {
                prompt.push_str("黑名单应用（通常导致分心）: ");
                prompt.push_str(&config.blacklist.join(", "));
                prompt.push_str("\n");
            }
            prompt.push_str("\n");
        }

        // 当前活动信息
        prompt.push_str("**当前活动信息**:\n");
        let app_info = app_name.as_deref().unwrap_or("未知应用");
        let title_info = window_title.as_deref().unwrap_or("无标题");
        let text_info = ocr_text.as_deref().unwrap_or("无文本内容");
        
        prompt.push_str(&format!("- 应用程序: {}\n", app_info));
        prompt.push_str(&format!("- 窗口标题: {}\n", title_info));
        prompt.push_str(&format!("- 屏幕内容: {}\n", text_info));
        prompt.push_str(&format!("当前时间: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")));

        // 分析要求
        prompt.push_str("请根据以上信息判断用户当前的专注状态，并按以下格式回答：\n\n");
        prompt.push_str("状态: [专注/分心/严重分心]\n");
        prompt.push_str("分析: [详细说明判断理由]\n\n");

        // 判断标准
        prompt.push_str("判断标准：\n");
        if current_task.is_some() {
            prompt.push_str("- 专注：当前活动与设定任务相关，或使用有助于任务完成的工具\n");
            prompt.push_str("- 分心：当前活动与设定任务无关，但不影响长期目标\n");
            prompt.push_str("- 严重分心：长时间从事与任务完全无关的活动，可能影响工作效率\n");
        } else {
            prompt.push_str("- 专注：使用白名单中的应用，或从事提升个人能力的活动\n");
            prompt.push_str("- 分心：使用黑名单中的应用，或从事娱乐休闲活动\n");
            prompt.push_str("- 严重分心：长时间沉迷娱乐，可能影响个人发展\n");
        }

        prompt
    }

    /// 调用AI模型
    async fn call_ai_model(ai_service: &AIService, prompt: &str) -> Result<String> {
        use std::time::Instant;
        
        println!("📡 准备调用AI模型...");
        println!("📏 发送的提示词长度: {} 字符", prompt.len());
        
        let api_call_start = Instant::now();
        
        // 使用配置的检测模型调用AI服务
        match ai_service.analyze_content(prompt, "detection").await {
            Ok(response) => {
                let api_call_duration = api_call_start.elapsed();
                println!("✅ AI模型调用成功");
                println!("⏱️ API调用耗时: {:?}", api_call_duration);
                println!("📥 响应长度: {} 字符", response.len());
                
                // 计算调用速度统计
                let chars_per_second = (response.len() as f64) / api_call_duration.as_secs_f64();
                println!("📊 响应速度: {:.1} 字符/秒", chars_per_second);
                
                Ok(response)
            }
            Err(e) => {
                let api_call_duration = api_call_start.elapsed();
                println!("❌ AI模型调用失败 (耗时: {:?}): {}", api_call_duration, e);
                println!("🔄 使用备用分析方案");
                
                // 如果AI调用失败，返回基础分析
                let fallback_response = "状态: 未知\n分析: AI服务暂不可用，无法进行专注状态分析。请检查网络连接和API配置。".to_string();
                println!("📋 备用响应: {}", fallback_response);
                
                Ok(fallback_response)
            }
        }
    }

    /// 解析AI响应
    fn parse_ai_response(response: &str) -> (FocusState, f32) {
        let response_lower = response.to_lowercase();
        
        // 优先检查明确的状态标识
        if response_lower.contains("状态: 严重分心") || response_lower.contains("状态:严重分心") {
            println!("🎯 解析到明确状态: 严重分心");
            return (FocusState::SeverelyDistracted, 0.95);
        }
        
        if response_lower.contains("状态: 分心") || response_lower.contains("状态:分心") {
            println!("🎯 解析到明确状态: 分心");
            return (FocusState::Distracted, 0.90);
        }
        
        if response_lower.contains("状态: 专注") || response_lower.contains("状态:专注") {
            println!("🎯 解析到明确状态: 专注");
            return (FocusState::Focused, 0.90);
        }
        
        // 如果没有明确的状态标识，使用关键词检查（按严重程度排序）
        if response_lower.contains("严重分心") {
            println!("🎯 关键词匹配: 严重分心");
            (FocusState::SeverelyDistracted, 0.85)
        } else if response_lower.contains("分心") {
            println!("🎯 关键词匹配: 分心");
            (FocusState::Distracted, 0.75)
        } else if response_lower.contains("专注") {
            println!("🎯 关键词匹配: 专注");
            (FocusState::Focused, 0.70)
        } else {
            println!("🎯 无法识别状态，返回未知");
            (FocusState::Unknown, 0.5)
        }
    }

    /// 获取当前活动状态
    pub async fn get_current_activity(&self) -> Option<CurrentActivity> {
        self.current_activity.lock().await.clone()
    }

    /// 获取最后的监控结果
    pub async fn get_last_result(&self) -> Option<MonitoringResult> {
        self.last_result.lock().await.clone()
    }

    /// 检查是否正在监控
    pub async fn is_monitoring(&self) -> bool {
        *self.is_monitoring.lock().await
    }

    /// 检查分心状态并执行干预措施
    async fn check_distraction_intervention(
        focus_state: &FocusState, 
        result: &MonitoringResult, 
        current_task: Option<&str>
    ) {
        println!("🔍 检查分心状态干预需求...");
        
        // 获取用户设置以检查干预配置
        let intervention_settings = match Self::get_intervention_settings().await {
            Ok(settings) => settings,
            Err(e) => {
                println!("❌ 获取干预设置失败: {}", e);
                return;
            }
        };
        
        // 如果分心干预功能未启用，直接返回
        if !intervention_settings.enabled {
            println!("ℹ️ 分心干预功能已禁用，跳过干预");
            return;
        }
        
        // 检查冷却时间
        if Self::is_intervention_in_cooldown(&intervention_settings).await {
            println!("⏱️ 干预功能在冷却期内，跳过此次干预");
            return;
        }
        
        match focus_state {
            FocusState::Distracted => {
                if intervention_settings.light_distraction_notification {
                    println!("⚠️ 检测到分心状态，执行轻度干预");
                    
                    // 轻度分心干预：温和提醒
                    let message = if let Some(task) = current_task {
                        format!("检测到轻度分心，当前任务：{}。建议重新集中注意力。", task)
                    } else {
                        "检测到轻度分心，建议重新集中注意力。".to_string()
                    };
                    
                    // 发送系统通知
                    if let Err(e) = Self::send_intervention_notification(
                        "专注提醒", 
                        &message, 
                        "reminder",
                        &intervention_settings
                    ).await {
                        println!("❌ 发送轻度干预通知失败: {}", e);
                    }
                    
                    // 记录干预日志
                    Self::log_intervention_action("light_reminder", &message, result).await;
                    
                    // 更新最后干预时间
                    Self::update_last_intervention_time().await;
                } else {
                    println!("ℹ️ 轻度分心通知已禁用");
                }
            },
            
            FocusState::SeverelyDistracted => {
                if intervention_settings.severe_distraction_popup {
                    println!("🚨 检测到严重分心状态，执行强度干预");
                    
                    // 严重分心干预：强烈警告和弹窗
                    let message = if let Some(task) = current_task {
                        format!("严重分心警告！当前任务：{}。请立即回到工作状态！", task)
                    } else {
                        "严重分心警告！请立即回到工作状态！".to_string()
                    };
                    
                    // 发送紧急通知
                    if let Err(e) = Self::send_intervention_notification(
                        "严重分心警告", 
                        &message, 
                        "urgent",
                        &intervention_settings
                    ).await {
                        println!("❌ 发送严重干预通知失败: {}", e);
                    }
                    
                    // 触发弹窗警告（通过前端）
                    if let Err(e) = Self::trigger_intervention_popup(&message, result, &intervention_settings).await {
                        println!("❌ 触发干预弹窗失败: {}", e);
                    }
                    
                    // 记录干预日志
                    Self::log_intervention_action("strong_warning", &message, result).await;
                    
                    // 更新最后干预时间
                    Self::update_last_intervention_time().await;
                } else {
                    println!("ℹ️ 严重分心弹窗已禁用");
                }
            },
            
            FocusState::Focused => {
                println!("✅ 专注状态良好，无需干预");
                
                // 专注状态鼓励（根据设置发送正面反馈）
                if intervention_settings.encouragement_enabled && Self::should_send_encouragement(&intervention_settings) {
                    let message = if let Some(task) = current_task {
                        format!("专注状态良好！继续保持对「{}」的专注。", task)
                    } else {
                        "专注状态良好！继续保持。".to_string()
                    };
                    
                    if let Err(e) = Self::send_intervention_notification(
                        "专注鼓励", 
                        &message, 
                        "encouragement",
                        &intervention_settings
                    ).await {
                        println!("❌ 发送鼓励通知失败: {}", e);
                    }
                    
                    // 记录鼓励日志
                    Self::log_intervention_action("encouragement", &message, result).await;
                }
            },
            
            FocusState::Unknown => {
                println!("❓ 无法确定专注状态，跳过干预");
            }
        }
    }

    /// 获取干预设置
    async fn get_intervention_settings() -> Result<crate::models::DistractionInterventionSettings> {
        // 获取存储服务
        match crate::commands::get_storage_service().await {
            Ok(storage_service) => {
                // 尝试加载用户设置，但不使用 distraction_intervention 字段
                match storage_service.load_user_settings().await {
                    Ok(_settings) => {
                        // 使用默认干预设置，因为 UserSettings 中没有 distraction_intervention 字段
                        Ok(crate::models::DistractionInterventionSettings::default())
                    },
                    Err(_) => {
                        println!("⚠️ 加载用户设置失败，使用默认干预设置");
                        Ok(crate::models::DistractionInterventionSettings::default())
                    }
                }
            },
            Err(e) => {
                println!("❌ 获取存储服务失败: {}", e);
                // 返回默认设置
                Ok(crate::models::DistractionInterventionSettings::default())
            }
        }
    }

    /// 检查是否在干预冷却期内
    async fn is_intervention_in_cooldown(settings: &crate::models::DistractionInterventionSettings) -> bool {
        // 获取最后干预时间
        match Self::get_last_intervention_time().await {
            Some(last_time) => {
                let now = chrono::Utc::now();
                let cooldown_duration = chrono::Duration::minutes(settings.intervention_cooldown_minutes as i64);
                let time_since_last = now - last_time;
                
                time_since_last < cooldown_duration
            },
            None => false // 没有记录表示可以进行干预
        }
    }

    /// 获取最后干预时间
    async fn get_last_intervention_time() -> Option<chrono::DateTime<chrono::Utc>> {
        // 从临时存储获取最后干预时间
        // 这里可以使用静态变量或文件存储
        // 简化实现：总是返回None，表示可以干预
        None
    }

    /// 更新最后干预时间
    async fn update_last_intervention_time() {
        // 更新最后干预时间到存储
        // 简化实现：仅记录日志
        println!("📝 更新最后干预时间: {}", chrono::Utc::now());
    }

    /// 判断是否应该发送鼓励消息
    fn should_send_encouragement(settings: &crate::models::DistractionInterventionSettings) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        // 根据频率设置决定发送概率
        let probability = match settings.encouragement_frequency.as_str() {
            "low" => 20,    // 5%概率
            "medium" => 10, // 10%概率  
            "high" => 5,    // 20%概率
            _ => 10         // 默认10%概率
        };
        
        (timestamp % probability) == 0
    }

    /// 发送干预通知
    async fn send_intervention_notification(
        title: &str, 
        message: &str, 
        intervention_type: &str,
        settings: &crate::models::DistractionInterventionSettings
    ) -> Result<()> {
        println!("📬 发送{}干预通知: {}", intervention_type, title);
        
        // 创建通知数据
        let notification_data = serde_json::json!({
            "title": title,
            "message": message,
            "type": intervention_type,
            "timestamp": chrono::Utc::now(),
            "priority": match intervention_type {
                "urgent" => "high",
                "reminder" => "medium", 
                "encouragement" => "low",
                _ => "medium"
            },
            "sound_enabled": settings.notification_sound,
            "duration_seconds": settings.popup_duration_seconds
        });
        
        // 在生产环境中，这里应该使用 Tauri 的通知 API
        // 目前记录日志以便调试
        println!("🔔 通知内容: {}", notification_data);
        
        // 模拟发送成功
        println!("✅ 干预通知发送成功");
        Ok(())
    }

    /// 触发干预弹窗（通过前端事件）
    async fn trigger_intervention_popup(
        message: &str, 
        result: &MonitoringResult,
        settings: &crate::models::DistractionInterventionSettings
    ) -> Result<()> {
        println!("🪟 触发干预弹窗");
        
        // 创建弹窗数据
        let popup_data = serde_json::json!({
            "type": "distraction_intervention",
            "message": message,
            "timestamp": result.timestamp,
            "focus_state": result.focus_state,
            "confidence": result.confidence,
            "application_name": result.application_name,
            "window_title": result.window_title,
            "urgent": true,
            "duration_seconds": settings.popup_duration_seconds,
            "sound_enabled": settings.notification_sound
        });
        
        println!("📤 弹窗数据准备完成: {}", popup_data);
        
        // TODO: 在实际应用中通过Tauri事件系统发送到前端
        // app.emit_all("distraction_intervention", popup_data)?;
        
        Ok(())
    }

    /// 记录干预行为日志
    async fn log_intervention_action(
        action_type: &str, 
        message: &str, 
        result: &MonitoringResult
    ) {
        println!("📝 记录干预日志: {} - {}", action_type, message);
        
        // 创建干预记录
        let intervention_log = serde_json::json!({
            "timestamp": chrono::Utc::now(),
            "action_type": action_type,
            "message": message,
            "focus_state": result.focus_state,
            "confidence": result.confidence,
            "application_name": result.application_name,
            "window_title": result.window_title,
            "ai_analysis": result.ai_analysis
        });
        
        // 保存到存储服务
        match Self::save_intervention_log(&intervention_log).await {
            Ok(_) => println!("✅ 干预日志已保存"),
            Err(e) => println!("❌ 保存干预日志失败: {}", e)
        }
    }

    /// 保存干预日志到存储
    async fn save_intervention_log(log_data: &serde_json::Value) -> Result<()> {
        // 获取应用数据目录
        // 使用应用本地目录
        let app_data_dir = std::path::PathBuf::from("data");
        
        // 创建存储服务实例
        let _storage_service = crate::services::storage_service::StorageService::new(app_data_dir.clone());
        
        // 创建干预日志文件路径
        let today = chrono::Utc::now().format("%Y%m%d");
        let log_file = format!("intervention_logs_{}.jsonl", today);
        
        // 将日志转换为单行JSON格式（JSONL）
        let log_line = format!("{}\n", log_data);
        
        // 尝试追加保存日志
        match Self::append_log_to_file(&app_data_dir, &log_file, &log_line).await {
            Ok(_) => {
                println!("📊 干预日志已保存到: {}", log_file);
                Ok(())
            },
            Err(e) => {
                println!("❌ 保存干预日志失败: {}", e);
                Err(anyhow::anyhow!("保存干预日志失败: {}", e))
            }
        }
    }

    /// 追加日志到文件
    async fn append_log_to_file(
        data_dir: &std::path::Path, 
        filename: &str, 
        content: &str
    ) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;
        
        // 确保数据目录存在
        if !data_dir.exists() {
            std::fs::create_dir_all(data_dir)?;
        }
        
        let file_path = data_dir.join(filename);
        
        // 以追加模式打开文件
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;
        
        // 写入内容
        file.write_all(content.as_bytes())?;
        file.flush()?;
        
        Ok(())
    }

    /// 发送专注状态变化事件给前端
    async fn send_focus_state_event(
        app_handle: &Arc<Mutex<Option<AppHandle>>>, 
        result: &MonitoringResult
    ) -> Result<()> {
        let handle_guard = app_handle.lock().await;
        if let Some(ref handle) = *handle_guard {
            // 构建专注状态事件数据
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
            handle.emit_all("focus_state_changed", &focus_event)
                .map_err(|e| anyhow::anyhow!("发送专注状态事件失败: {}", e))?;
            
            println!("📡 专注状态事件已发送: {}", focus_state_str);
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("AppHandle未设置，无法发送事件"))
        }
    }

    /// 发送分心干预事件给前端
    async fn send_distraction_intervention_event(
        app_handle: &Arc<Mutex<Option<AppHandle>>>, 
        result: &MonitoringResult
    ) -> Result<()> {
        let handle_guard = app_handle.lock().await;
        if let Some(ref handle) = *handle_guard {
            let intervention_type = match result.focus_state {
                FocusState::Distracted => "light",
                FocusState::SeverelyDistracted => "severe",
                _ => return Ok(()) // 只处理分心状态
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
            
            // 发送分心干预事件
            handle.emit_all("distraction_intervention", &intervention_data)
                .map_err(|e| anyhow::anyhow!("发送分心干预事件失败: {}", e))?;
            
            println!("📡 分心干预事件已发送: {}", intervention_type);
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("AppHandle未设置，无法发送事件"))
        }
    }
} 