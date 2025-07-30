/**
 * Tauri API 集成模块
 * 封装与后端的通信接口
 */

import { invoke } from '@tauri-apps/api/tauri';

// API调用封装
async function invokeCommand(command, params = {}) {
    try {
        const result = await invoke(command, params);
        return result;
    } catch (error) {
        console.error(`调用命令 ${command} 失败:`, error);
        
        // 根据错误类型提供更详细的错误信息
        if (error.message && error.message.includes('AI配置')) {
            throw new Error('AI服务配置错误，请检查API设置');
        } else if (error.message && error.message.includes('监控数据')) {
            throw new Error('监控服务未启动或数据不足');
        } else if (error.message && error.message.includes('存储服务')) {
            throw new Error('数据存储服务异常，请重试');
        } else {
            throw new Error(`后端服务错误: ${error.message || error}`);
        }
    }
}

// 错误处理辅助函数
function handleApiError(error, context = '') {
    console.error(`API调用错误 [${context}]:`, error);
    
    const errorMessage = error.message || '未知错误';
    
    // 显示用户友好的错误信息
    if (window.showErrorNotification) {
        window.showErrorNotification(`${context ? context + ': ' : ''}${errorMessage}`);
    } else {
        alert(`操作失败: ${errorMessage}`);
    }
    
    // 返回标准错误格式而不是null
    return {
        success: false,
        message: errorMessage,
        response_time_ms: 0
    };
}

// 安全的API调用包装器
async function safeInvoke(command, params = {}, context = '') {
    try {
        return await invokeCommand(command, params);
    } catch (error) {
        return handleApiError(error, context);
    }
}

// API 接口定义
export const TauriAPI = {
    // 应用状态管理
    async getAppStatus() {
        return await safeInvoke('get_app_status', {}, '获取应用状态');
    },
    
    async initializeApp() {
        return await safeInvoke('initialize_app', {}, '初始化应用');
    },
    
    // 用户设置管理
    async saveUserSettings(settings) {
        return await safeInvoke('save_user_settings', { settings }, '保存用户设置');
    },
    
    async loadUserSettings() {
        return await safeInvoke('load_user_settings', {}, '加载用户设置');
    },
    
    // 任务管理
    async saveTask(taskParam) {
        // 兼容两种调用方式：直接传字符串或传对象
        const taskData = typeof taskParam === 'string' 
            ? { text: taskParam }
            : taskParam;
        return await safeInvoke('save_task', { task: taskData }, '保存任务');
    },
    
    async getTasks(date = null) {
        return await safeInvoke('get_tasks', { date }, '获取任务列表');
    },
    
    async updateTaskStatus(taskId, completed) {
        return await safeInvoke('update_task_status', { task_id: taskId, completed }, '更新任务状态');
    },
    
    async deleteTask(taskId) {
        return await safeInvoke('delete_task', { task_id: taskId }, '删除任务');
    },
    
    // 系统监控
    async startMonitoring() {
        return await safeInvoke('start_monitoring', {}, '启动监控');
    },
    
    async stopMonitoring() {
        return await safeInvoke('stop_monitoring', {}, '停止监控');
    },
    
    async getCurrentActivity() {
        return await safeInvoke('get_current_activity', {}, '获取当前活动');
    },
    
    async triggerMonitoringCheck() {
        return await safeInvoke('trigger_monitoring_check', {}, '触发监控检查');
    },
    
    // 专注计时器
    async startFocusTimer(taskName = null, duration = 25) {
        return await safeInvoke('start_focus_timer', { 
            task_name: taskName, 
            duration 
        }, '启动专注计时器');
    },
    
    async pauseFocusTimer() {
        return await safeInvoke('pause_focus_timer', {}, '暂停专注计时器');
    },
    
    async stopFocusTimer() {
        return await safeInvoke('stop_focus_timer', {}, '停止专注计时器');
    },
    
    async getTimerStatus() {
        return await safeInvoke('get_timer_status', {}, '获取计时器状态');
    },
    
    // 数据统计
    async getTodayStatistics() {
        return await safeInvoke('get_today_statistics', {}, '获取今日统计');
    },
    
    async getFocusHistory(days = 7) {
        return await safeInvoke('get_focus_history', { days }, '获取专注历史');
    },
    
    // AI 配置管理
    async saveAIConfig(config) {
        return await safeInvoke('save_ai_config', { config }, '保存AI配置');
    },
    
    async loadAIConfig() {
        return await safeInvoke('load_ai_config', {}, '加载AI配置');
    },
    
    async testAIAPI(config) {
        return await safeInvoke('test_ai_api', { config }, 'AI API连接测试');
    },
    
    async getAvailableModels(config) {
        return await safeInvoke('get_available_models', { config }, '获取可用模型');
    },
    
    async refreshModels(config) {
        return await safeInvoke('refresh_models', { config }, '刷新模型列表');
    },
    
    // 监控配置管理
    async saveMonitoringConfig(config) {
        return await safeInvoke('save_monitoring_config', { config }, '保存监控配置');
    },
    
    async loadMonitoringConfig() {
        return await safeInvoke('load_monitoring_config', {}, '加载监控配置');
    },
    
    async getCurrentFocusState() {
        return await safeInvoke('get_current_focus_state', {}, '获取当前专注状态');
    },
    
    async updateMonitoringInterval(intervalMinutes) {
        return await safeInvoke('update_monitoring_interval', { 
            interval_minutes: intervalMinutes 
        }, '更新监控间隔');
    },
    
    // 报告生成
    async generateDailyReport(date) {
        return await safeInvoke('generate_daily_report', { date }, '生成日报告');
    },
    
    async generateWeeklyReport(weekStart) {
        return await safeInvoke('generate_weekly_report', { week_start: weekStart }, '生成周报告');
    },
    
    async getReportList(reportType = 'daily', limit = 30) {
        return await safeInvoke('get_report_list', { 
            report_type: reportType, 
            limit 
        }, '获取报告列表');
    },
    
    async deleteReport(reportId) {
        return await safeInvoke('delete_report', { report_id: reportId }, '删除报告');
    },
    
    async exportReportData(dateRange, format = 'json') {
        return await safeInvoke('export_report_data', { 
            date_range: dateRange, 
            format 
        }, '导出报告数据');
    }
};

// 事件处理（如果需要）
export const TauriEvents = {
    // 监听Tauri事件
    async listen(event, handler) {
        try {
            const { listen } = await import('@tauri-apps/api/event');
            return await listen(event, handler);
        } catch (error) {
            console.warn('Tauri事件系统不可用:', error);
            return null;
        }
    },
    
    // 发送事件
    async emit(event, payload) {
        try {
            const { emit } = await import('@tauri-apps/api/event');
            return await emit(event, payload);
        } catch (error) {
            console.warn('Tauri事件系统不可用:', error);
            return null;
        }
    },
    
    // 专注状态变化事件
    async onFocusStateChanged(handler) {
        return await this.listen('focus_state_changed', handler);
    },
    
    // 计时器更新事件
    async onTimerTick(handler) {
        return await this.listen('timer_tick', handler);
    },
    
    // 分心干预事件
    async onDistractionIntervention(handler) {
        return await this.listen('distraction_intervention', handler);
    }
};

// 初始化检查
document.addEventListener('DOMContentLoaded', async () => {
    try {
        // 测试 Tauri 连接
        await invokeCommand('get_app_status');
        console.log('Tauri API已就绪');
        
        // 初始化应用
        try {
            await TauriAPI.initializeApp();
            console.log('应用初始化完成');
        } catch (error) {
            console.error('应用初始化失败:', error);
        }
    } catch (error) {
        console.warn('应用未在Tauri环境中运行，某些功能可能不可用:', error);
        
        // 显示环境警告
        const warningDiv = document.createElement('div');
        warningDiv.innerHTML = `
            <div style="background: #ff6b6b; color: white; padding: 10px; text-align: center; position: fixed; top: 0; left: 0; right: 0; z-index: 9999;">
                ⚠️ 警告：应用未在正确的环境中运行，请使用 <code>npm run tauri:dev</code> 启动
            </div>
        `;
        document.body.appendChild(warningDiv);
    }
});

export default TauriAPI;