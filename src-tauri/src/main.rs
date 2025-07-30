// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod services;

use commands::*;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // 应用状态管理
            get_app_status,
            initialize_app,
            
            // 用户设置管理
            save_user_settings,
            load_user_settings,
            
            // 任务管理
            save_task,
            get_tasks,
            update_task_status,
            delete_task,
            
            // 系统监控
            start_monitoring,
            stop_monitoring,
            get_current_activity,
            
            // 专注计时器
            start_focus_timer,
            pause_focus_timer,
            stop_focus_timer,
            get_timer_status,
            
            // 数据统计
            get_today_statistics,
            get_focus_history,
            
            // AI 配置管理
            save_ai_config,
            load_ai_config,
            test_ai_api,
            get_available_models,
            refresh_models,
            
            // 监控配置管理
            save_monitoring_config,
            load_monitoring_config,
            get_current_focus_state,
            update_monitoring_interval,
            trigger_monitoring_check,
            
            // 报告生成管理
            generate_daily_report,
            generate_weekly_report,
            get_report_list,
            delete_report,
            export_report_data,
            
            // 数据管理
            cleanup_old_data,
            get_storage_usage,
            optimize_storage,
            backup_data,
            restore_data
        ])
        .setup(|app| {
            // 应用启动时的初始化
            println!("My Focus 应用正在启动...");
            
            // 这里可以添加数据库初始化等逻辑
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
} 