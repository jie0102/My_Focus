/**
 * My Focus - 专注度应用前端脚本
 */

// 导入Tauri API
import { TauriAPI, TauriEvents } from './tauri-api.js';

// 全局状态变量
let isMonitoring = false;
let pauseTimer = null;
let pauseCountdown = 0;
let currentFocusState = 'idle'; // idle, focused, distracted, severely_distracted
let currentSelectedTask = null;

// 模态框事件处理器
let modalKeyPressHandler = null;
let modalOutsideClickHandler = null;

// 状态配置
const statusConfig = {
    idle: {
        icon: 'fas fa-pause',
        text: '未开始监控',
        color: 'gray',
        bgColor: 'bg-gray-500/10',
        borderColor: 'border-gray-500',
        textColor: 'text-gray-400',
        statusTextColor: 'text-white'
    },
    focused: {
        icon: 'fas fa-brain',
        text: '深度专注中',
        color: 'green',
        bgColor: 'bg-green-500/10',
        borderColor: 'border-green-500',
        textColor: 'text-green-400',
        statusTextColor: 'text-white',
        animation: 'animate-pulse'
    },
    distracted: {
        icon: 'fas fa-exclamation-triangle',
        text: '轻度分心',
        color: 'yellow',
        bgColor: 'bg-yellow-500/10',
        borderColor: 'border-yellow-500',
        textColor: 'text-yellow-400',
        statusTextColor: 'text-white'
    },
    severely_distracted: {
        icon: 'fas fa-times-circle',
        text: '严重分心',
        color: 'red',
        bgColor: 'bg-red-500/10',
        borderColor: 'border-red-500',
        textColor: 'text-red-400',
        statusTextColor: 'text-white',
        animation: 'animate-pulse'
    }
};

// 页面加载完成后初始化
document.addEventListener('DOMContentLoaded', function () {
    initApp();
    initTabs();
    initTimer();
    initTasks();
    initSettings();
    initTauriEvents();
    initDashboardControls();
});

/**
 * 初始化应用
 */
async function initApp() {
    try {
        console.log('初始化My Focus应用...');
        
        // 首先加载并应用保存的主题
        loadSavedTheme();
        
        // 初始化后端
        await TauriAPI.initializeApp();
        
        // 获取应用状态
        const status = await TauriAPI.getAppStatus();
        console.log('应用状态:', status);
        
        // 加载今日统计数据
        updateDashboardWithRealData();
        
    } catch (error) {
        console.error('应用初始化失败:', error);
    }
}

/**
 * 初始化Tauri事件监听
 */
async function initTauriEvents() {
    try {
        // 监听专注状态变化
        await TauriEvents.onFocusStateChanged((event) => {
            console.log('专注状态变化:', event.payload);
            updateFocusStatus(event.payload);
        });
        
        // 监听计时器事件
        await TauriEvents.onTimerTick((event) => {
            console.log('计时器更新:', event.payload);
            updateTimerDisplay(event.payload);
        });
        
        // 监听分心干预事件
        await TauriEvents.onDistractionIntervention((event) => {
            console.log('分心干预:', event.payload);
            showDistractionIntervention(event.payload);
        });
        
    } catch (error) {
        console.error('事件监听初始化失败:', error);
    }
}

/**
 * 初始化标签页功能
 */
function initTabs() {
    const tabs = document.querySelectorAll('.tab-button');
    const pages = document.querySelectorAll('.page-content');

    tabs.forEach(tab => {
        tab.addEventListener('click', () => {
            const targetId = tab.dataset.target;
            const targetPage = document.getElementById(targetId);

            // 移除所有激活状态
            tabs.forEach(t => t.classList.remove('tab-active'));
            pages.forEach(p => p.classList.add('hidden'));

            // 激活当前标签和页面
            tab.classList.add('tab-active');
            if (targetPage) {
                targetPage.classList.remove('hidden');
            }

            // 触发页面切换事件
            onPageSwitch(targetId);
        });
    });
}

/**
 * 页面切换时的处理
 */
function onPageSwitch(pageId) {
    switch (pageId) {
        case 'timer':
            updateTimerDisplay();
            updateTaskSelector(); // 切换到计时器页面时同步任务
            break;
        case 'reports':
            loadReports();
            break;
        case 'fatigue':
            updateFatigueLevel();
            break;
        case 'settings':
            loadSettings();
            break;
        default:
            updateDashboard();
    }
}

/**
 * 初始化计时器功能
 */
function initTimer() {
    let timerInterval = null;
    let currentTime = 25 * 60; // 25分钟，单位秒
    let isRunning = false;
    let isPaused = false;
    let focusDuration = 25 * 60; // 专注时长（秒）
    let shortBreakDuration = 5 * 60; // 短休息时长（秒）
    let longBreakDuration = 15 * 60; // 长休息时长（秒）

    const timerDisplay = document.getElementById('timer-display');
    const timerStatus = document.getElementById('timer-status');
    const mainButton = document.getElementById('timer-main-btn');
    const resetButton = document.getElementById('timer-reset-btn');
    const stopButton = document.getElementById('timer-stop-btn');
    const applySettingsButton = document.getElementById('apply-timer-settings');

    // 更新显示
    function updateDisplay() {
        const minutes = Math.floor(currentTime / 60);
        const seconds = currentTime % 60;
        if (timerDisplay) {
            timerDisplay.textContent = `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
        }
    }

    // 更新按钮状态
    function updateButtonState() {
        if (!isRunning && !isPaused) {
            // 初始状态 - 显示开始按钮
            mainButton.innerHTML = '<i class="fas fa-play text-5xl"></i>';
            mainButton.className = 'bg-green-600 hover:bg-green-700 text-white font-bold w-28 h-28 rounded-full flex items-center justify-center transition-transform duration-200 hover:scale-105 shadow-2xl shadow-green-900/50';
            timerStatus.textContent = '准备开始专注';
            timerStatus.className = 'text-lg font-semibold text-gray-400';
        } else if (isRunning) {
            // 运行状态 - 显示暂停按钮
            mainButton.innerHTML = '<i class="fas fa-pause text-5xl"></i>';
            mainButton.className = 'bg-blue-600 hover:bg-blue-700 text-white font-bold w-28 h-28 rounded-full flex items-center justify-center transition-transform duration-200 hover:scale-105 shadow-2xl shadow-blue-900/50';
            timerStatus.textContent = '专注中';
            timerStatus.className = 'text-lg font-semibold text-blue-400';
        } else if (isPaused) {
            // 暂停状态 - 显示继续按钮
            mainButton.innerHTML = '<i class="fas fa-play text-5xl"></i>';
            mainButton.className = 'bg-green-600 hover:bg-green-700 text-white font-bold w-28 h-28 rounded-full flex items-center justify-center transition-transform duration-200 hover:scale-105 shadow-2xl shadow-green-900/50';
            timerStatus.textContent = '已暂停';
            timerStatus.className = 'text-lg font-semibold text-yellow-400';
        }
    }

    // 开始/暂停/继续计时器
    function toggleTimer() {
        if (!isRunning && !isPaused) {
            // 开始计时器
            startTimer();
        } else if (isRunning) {
            // 暂停计时器
            pauseTimer();
        } else if (isPaused) {
            // 继续计时器
            resumeTimer();
        }
    }

    // 开始计时器
    function startTimer() {
        timerInterval = setInterval(() => {
            currentTime--;
            updateDisplay();
            
            if (currentTime <= 0) {
                clearInterval(timerInterval);
                isRunning = false;
                isPaused = false;
                updateButtonState();
                showNotification('专注时间完成！', '恭喜您完成了一个专注时段。', 'success', true);
                
                // 重置到设定的专注时长
                currentTime = focusDuration;
                updateDisplay();
            }
        }, 1000);
        
        isRunning = true;
        isPaused = false;
        updateButtonState();
        showNotification('开始专注', '专注计时已开始！', 'info', true);
    }

    // 暂停计时器
    function pauseTimer() {
        clearInterval(timerInterval);
        isRunning = false;
        isPaused = true;
        updateButtonState();
    }

    // 继续计时器
    function resumeTimer() {
        startTimer(); // 重新开始计时
    }

    // 重置计时器
    function resetTimer() {
        clearInterval(timerInterval);
        currentTime = focusDuration;
        isRunning = false;
        isPaused = false;
        updateDisplay();
        updateButtonState();
        showNotification('计时器已重置', '计时器已重置到初始状态');
    }

    // 停止计时器
    function stopTimer() {
        clearInterval(timerInterval);
        currentTime = focusDuration;
        isRunning = false;
        isPaused = false;
        updateDisplay();
        updateButtonState();
        showNotification('计时器已停止', '计时器已停止并重置');
    }

    // 应用设置
    function applySettings() {
        const focusInput = document.getElementById('focus-duration');
        const shortBreakInput = document.getElementById('short-break');
        const longBreakInput = document.getElementById('long-break');

        if (focusInput && shortBreakInput && longBreakInput) {
            const newFocusDuration = parseInt(focusInput.value) * 60;
            const newShortBreak = parseInt(shortBreakInput.value) * 60;
            const newLongBreak = parseInt(longBreakInput.value) * 60;

            // 验证输入值
            if (newFocusDuration > 0 && newShortBreak > 0 && newLongBreak > 0) {
                focusDuration = newFocusDuration;
                shortBreakDuration = newShortBreak;
                longBreakDuration = newLongBreak;

                // 如果计时器未运行，更新当前时间
                if (!isRunning && !isPaused) {
                    currentTime = focusDuration;
                    updateDisplay();
                }

                showNotification('设置已应用', '计时器设置已成功更新！');
                
                // 保存设置到本地存储
                const timerSettings = {
                    focusDuration: focusDuration / 60,
                    shortBreakDuration: shortBreakDuration / 60,
                    longBreakDuration: longBreakDuration / 60
                };
                localStorage.setItem('timerSettings', JSON.stringify(timerSettings));
            } else {
                showNotification('设置无效', '请输入有效的时间值（大于0）');
            }
        }
    }

    // 加载保存的设置
    function loadTimerSettings() {
        const savedSettings = localStorage.getItem('timerSettings');
        if (savedSettings) {
            const settings = JSON.parse(savedSettings);
            const focusInput = document.getElementById('focus-duration');
            const shortBreakInput = document.getElementById('short-break');
            const longBreakInput = document.getElementById('long-break');

            if (focusInput) focusInput.value = settings.focusDuration || 25;
            if (shortBreakInput) shortBreakInput.value = settings.shortBreakDuration || 5;
            if (longBreakInput) longBreakInput.value = settings.longBreakDuration || 15;

            // 应用保存的设置
            focusDuration = (settings.focusDuration || 25) * 60;
            shortBreakDuration = (settings.shortBreakDuration || 5) * 60;
            longBreakDuration = (settings.longBreakDuration || 15) * 60;
            currentTime = focusDuration;
        }
    }

    // 绑定事件
    if (mainButton) mainButton.addEventListener('click', toggleTimer);
    if (resetButton) resetButton.addEventListener('click', resetTimer);
    if (stopButton) stopButton.addEventListener('click', stopTimer);
    if (applySettingsButton) applySettingsButton.addEventListener('click', applySettings);

    // 初始化
    loadTimerSettings();
    updateDisplay();
    updateButtonState();
    updateTaskSelector(); // 初始化任务选择器
}

/**
 * 初始化任务管理
 */
function initTasks() {
    const addTaskButton = document.getElementById('add-task-btn');
    const taskInput = document.getElementById('task-input');
    const taskList = document.getElementById('task-list');

    if (addTaskButton && taskInput) {
        addTaskButton.addEventListener('click', addNewTask);
        taskInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                addNewTask();
            }
        });
        console.log('任务添加功能已初始化'); // 添加调试日志
    } else {
        console.error('无法找到任务添加元素:', { 
            addTaskButton: !!addTaskButton, 
            taskInput: !!taskInput 
        });
    }

    async function addNewTask() {
        const taskText = taskInput.value.trim();
        if (!taskText) return;

        try {
            // 保存到后端 - 直接传递字符串参数
            const newTask = await TauriAPI.saveTask(taskText);
            console.log('新任务已保存:', newTask);
            
            // 隐藏空任务提示
            hideEmptyTaskMessage();
            
            // 创建任务元素
            const taskItem = createTaskElement(newTask);
            taskList.appendChild(taskItem);
            
            taskInput.value = '';
            showNotification('任务添加成功', `已添加任务: ${taskText}`);
            
            // 同步任务到计时器界面
            updateTaskSelector();
            
            // 检查当前选中的任务是否还存在
            validateCurrentTask();
            
        } catch (error) {
            console.error('任务保存失败:', error);
            showNotification('任务添加失败', '请稍后重试: ' + error);
        }
    }

    /**
     * 创建任务元素
     */
    function createTaskElement(task) {
        console.log('创建任务元素，接收到的task对象:', task);
        
        if (!task || typeof task !== 'object') {
            console.error('无效的任务对象:', task);
            return null;
        }
        
        const taskItem = document.createElement('li');
        taskItem.className = 'flex items-center justify-between p-2 rounded-md hover:bg-gray-800/50 group';
        taskItem.setAttribute('data-task-id', task.id || 'unknown');
        
        // 兼容不同的字段名：text 或 title，并添加调试信息
        let taskText = '';
        if (task.text) {
            taskText = typeof task.text === 'string' ? task.text : '未命名任务';
        } else if (task.title) {
            taskText = typeof task.title === 'string' ? task.title : '未命名任务';
        } else {
            taskText = '未命名任务';
        }
        
        console.log('最终显示的任务文本:', taskText);
        
        taskItem.innerHTML = `
            <div class="flex items-center flex-grow">
                <input id="${task.id || 'unknown'}" type="checkbox" class="h-4 w-4 rounded bg-gray-600 border-gray-500 text-blue-500 focus:ring-blue-600" ${task.completed ? 'checked' : ''}>
                <label for="${task.id || 'unknown'}" class="ml-3 text-gray-300 truncate cursor-pointer">${taskText}</label>
            </div>
            <button class="text-red-400 hover:text-red-300 opacity-0 group-hover:opacity-100 transition-opacity ml-2 p-1" data-delete-task="${task.id || 'unknown'}">
                <i class="fas fa-trash text-sm"></i>
            </button>
        `;

        // 添加状态变化监听
        const checkbox = taskItem.querySelector('input[type="checkbox"]');
        if (checkbox) {
            checkbox.addEventListener('change', async (e) => {
                try {
                    await TauriAPI.updateTaskStatus(task.id, e.target.checked);
                    console.log(`任务 ${task.id} 状态已更新为: ${e.target.checked}`);
                    
                    // 同步任务状态到计时器界面
                    updateTaskSelector();
                    
                    // 如果完成的是当前选中的任务，清除当前任务
                    if (e.target.checked && currentSelectedTask === task.id) {
                        clearCurrentTask();
                    }
                    
                } catch (error) {
                    console.error('任务状态更新失败:', error);
                    // 还原状态
                    e.target.checked = !e.target.checked;
                    showNotification('更新失败', '任务状态更新失败');
                }
            });
        }

        // 添加删除按钮监听
        const deleteBtn = taskItem.querySelector(`[data-delete-task="${task.id || 'unknown'}"]`);
        if (deleteBtn) {
            deleteBtn.addEventListener('click', async (e) => {
                e.preventDefault();
                e.stopPropagation();
                await deleteTask(task.id);
            });
        }

        return taskItem;
    }

    // 删除任务到后端
    async function deleteTaskFromBackend(taskId) {
        try {
            await TauriAPI.deleteTask(taskId);
            console.log(`任务 ${taskId} 已从后端删除`);
        } catch (error) {
            console.error('从后端删除任务失败:', error);
            throw error;
        }
    }

    // 从本地存储保存任务（备用）
    function saveTasks() {
        const tasks = [];
        document.querySelectorAll('[data-task-id]').forEach(li => {
            const checkbox = li.querySelector('input[type="checkbox"]');
            const label = li.querySelector('label');
            if (checkbox && label) {
                tasks.push({
                    id: checkbox.id,
                    text: label.textContent,
                    completed: checkbox.checked
                });
            }
        });
        localStorage.setItem('myFocusTasks', JSON.stringify(tasks));
    }

    // 从后端加载任务
    async function loadTasks() {
        try {
            const tasks = await TauriAPI.getTasks();
            console.log('从后端加载的任务:', tasks);
            updateTasksList(tasks);
        } catch (error) {
            console.error('从后端加载任务失败:', error);
            // 如果后端加载失败，尝试从本地存储加载
            loadTasksFromLocal();
        }
    }

    // 从本地存储加载任务（备用方案）
    function loadTasksFromLocal() {
        const savedTasks = localStorage.getItem('myFocusTasks');
        if (savedTasks) {
            const tasks = JSON.parse(savedTasks);
            updateTasksList(tasks);
        }
    }

    // 初始化时检查是否需要显示空任务提示
    checkAndShowEmptyTaskMessage();
    loadTasks();
    
    // 初始化任务选择器
    updateTaskSelector();
}

/**
 * 初始化设置功能
 */
function initSettings() {
    const saveButton = document.querySelector('#settings .bg-blue-600');
    
    if (saveButton) {
        saveButton.addEventListener('click', saveSettings);
    }

    // 初始化白名单黑名单管理
    initWhitelistBlacklistManagement();
    
    // 初始化帮助提示
    initHelpTooltips();
    
    // 初始化分心干预设置帮助提示
    initInterventionTooltips();
    
    // 初始化AI模型设置
    initAIModelSettings();
    
    // 初始化主题切换功能
    initThemeToggle();

    function saveSettings() {
        const settings = {
            theme: getCurrentTheme(),
            whitelist: getWhitelistItems(),
            blacklist: getBlacklistItems(),
            autostart: document.getElementById('autostart')?.checked || false,
            fatigue_notify: document.getElementById('fatigue-notify')?.checked || false,
            focus_duration: 25,
            short_break: 5,
            long_break: 15,
            
            // 分心干预设置
            distraction_intervention: {
                enabled: document.getElementById('distraction-intervention-enabled')?.checked || true,
                light_distraction_notification: document.getElementById('light-distraction-notification')?.checked || true,
                severe_distraction_popup: document.getElementById('severe-distraction-popup')?.checked || true,
                encouragement_enabled: document.getElementById('encouragement-enabled')?.checked || true,
                intervention_cooldown_minutes: parseInt(document.getElementById('intervention-cooldown')?.value || '5'),
                notification_sound: document.getElementById('intervention-sound')?.checked || true,
                popup_duration_seconds: parseInt(document.getElementById('popup-duration')?.value || '10'),
                encouragement_frequency: document.getElementById('encouragement-frequency')?.value || 'medium'
            }
        };

        // 保存用户设置到后端
        saveUserSettingsToBackend(settings);
        
        // 注意：不再自动保存AI配置，需要用户点击"保存设置"按钮
        
        // 显示保存成功提示（但提醒用户还需保存AI配置）
        showNotification('用户设置已保存', '用户偏好设置已保存，如有AI配置更改请点击"保存设置"按钮', 'success', false);
    }

/**
 * 处理保存设置按钮点击事件
 */
async function handleSaveSettings() {
    const saveBtn = document.getElementById('save-settings-btn');
    if (!saveBtn) return;
    
    // 更新按钮状态
    const originalText = saveBtn.innerHTML;
    saveBtn.innerHTML = '<i class="fas fa-spinner fa-spin mr-2"></i>正在保存...';
    saveBtn.disabled = true;
    
    try {
        // 保存所有设置
        await saveAllSettings();
        
        // 保存AI配置
        await saveAIConfig();
        
        // 成功提示
        saveBtn.innerHTML = '<i class="fas fa-check mr-2"></i>保存成功';
        saveBtn.classList.remove('bg-blue-600', 'hover:bg-blue-700');
        saveBtn.classList.add('bg-green-600', 'hover:bg-green-700');
        
        showNotification('设置已保存', '所有设置已成功保存到本地', 'success', false);
        
        // 2秒后恢复按钮状态
        setTimeout(() => {
            saveBtn.innerHTML = originalText;
            saveBtn.disabled = false;
            saveBtn.classList.remove('bg-green-600', 'hover:bg-green-700');
            saveBtn.classList.add('bg-blue-600', 'hover:bg-blue-700');
        }, 2000);
        
    } catch (error) {
        console.error('保存设置失败:', error);
        
        // 错误提示
        saveBtn.innerHTML = '<i class="fas fa-exclamation-triangle mr-2"></i>保存失败';
        saveBtn.classList.remove('bg-blue-600', 'hover:bg-blue-700');
        saveBtn.classList.add('bg-red-600', 'hover:bg-red-700');
        
        showNotification('保存失败', `设置保存失败: ${error.message || error}`, 'error', false);
        
        // 3秒后恢复按钮状态
        setTimeout(() => {
            saveBtn.innerHTML = originalText;
            saveBtn.disabled = false;
            saveBtn.classList.remove('bg-red-600', 'hover:bg-red-700');
            saveBtn.classList.add('bg-blue-600', 'hover:bg-blue-700');
        }, 3000);
    }
}

    /**
     * 保存用户设置到后端
     */
    async function saveUserSettingsToBackend(settings) {
        try {
            await TauriAPI.saveUserSettings(settings);
            console.log('用户设置已保存到后端:', settings);
            
            // 同时保存到本地存储作为备份
            localStorage.setItem('myFocusSettings', JSON.stringify(settings));
        } catch (error) {
            console.error('保存用户设置到后端失败:', error);
            showNotification('保存失败', '设置保存到后端失败，但已保存到本地');
            // 如果后端保存失败，至少保存到本地存储
            localStorage.setItem('myFocusSettings', JSON.stringify(settings));
        }
    }

    function loadSettings() {
        // 首先尝试从后端加载设置
        loadUserSettingsFromBackend();
    }

    /**
     * 从后端加载用户设置
     */
    async function loadUserSettingsFromBackend() {
        try {
            const settings = await TauriAPI.loadUserSettings();
            console.log('从后端加载的用户设置:', settings);
            
            // 恢复白名单黑名单
            if (settings.whitelist && Array.isArray(settings.whitelist)) {
                loadWhitelistItems(settings.whitelist);
            } else {
                loadWhitelistItems([]);
            }
            
            if (settings.blacklist && Array.isArray(settings.blacklist)) {
                loadBlacklistItems(settings.blacklist);
            } else {
                loadBlacklistItems([]);
            }
            
            // 恢复主题设置
            if (settings.theme) {
                applyTheme(settings.theme);
            }
            
            // 恢复其他设置值
            if (document.getElementById('autostart')) {
                document.getElementById('autostart').checked = settings.autostart || false;
            }
            if (document.getElementById('fatigue-notify')) {
                document.getElementById('fatigue-notify').checked = settings.fatigue_notify || false;
            }
            
            // 恢复分心干预设置
            if (settings.distraction_intervention) {
                const intervention = settings.distraction_intervention;
                
                if (document.getElementById('distraction-intervention-enabled')) {
                    document.getElementById('distraction-intervention-enabled').checked = intervention.enabled !== false;
                }
                if (document.getElementById('light-distraction-notification')) {
                    document.getElementById('light-distraction-notification').checked = intervention.light_distraction_notification !== false;
                }
                if (document.getElementById('severe-distraction-popup')) {
                    document.getElementById('severe-distraction-popup').checked = intervention.severe_distraction_popup !== false;
                }
                if (document.getElementById('encouragement-enabled')) {
                    document.getElementById('encouragement-enabled').checked = intervention.encouragement_enabled !== false;
                }
                if (document.getElementById('intervention-cooldown')) {
                    document.getElementById('intervention-cooldown').value = intervention.intervention_cooldown_minutes || 5;
                }
                if (document.getElementById('intervention-sound')) {
                    document.getElementById('intervention-sound').checked = intervention.notification_sound !== false;
                }
                if (document.getElementById('popup-duration')) {
                    document.getElementById('popup-duration').value = intervention.popup_duration_seconds || 10;
                }
                if (document.getElementById('encouragement-frequency')) {
                    document.getElementById('encouragement-frequency').value = intervention.encouragement_frequency || 'medium';
                }
            } else {
                // 如果没有分心干预设置，使用默认值
                console.log('使用默认分心干预设置');
                restoreDefaultInterventionSettings();
            }
            
        } catch (error) {
            console.error('从后端加载用户设置失败:', error);
            // 如果后端加载失败，尝试从本地存储加载
            loadSettingsFromLocal();
        }
    }

    /**
     * 从本地存储加载设置（备用方案）
     */
    function loadSettingsFromLocal() {
        const savedSettings = localStorage.getItem('myFocusSettings');
        if (savedSettings) {
            const settings = JSON.parse(savedSettings);
            
            // 恢复白名单黑名单
            if (settings.whitelist && Array.isArray(settings.whitelist)) {
                loadWhitelistItems(settings.whitelist);
            } else if (typeof settings.whitelist === 'string') {
                // 兼容旧版本的字符串格式
                const items = settings.whitelist.split('\n').filter(item => item.trim());
                loadWhitelistItems(items);
            } else {
                loadWhitelistItems([]);
            }
            
            if (settings.blacklist && Array.isArray(settings.blacklist)) {
                loadBlacklistItems(settings.blacklist);
            } else if (typeof settings.blacklist === 'string') {
                // 兼容旧版本的字符串格式
                const items = settings.blacklist.split('\n').filter(item => item.trim());
                loadBlacklistItems(items);
            } else {
                loadBlacklistItems([]);
            }
            
            // 恢复其他设置值
            const otherFields = ['autostart', 'fatigueNotify'];
            otherFields.forEach(key => {
                const element = document.getElementById(key.replace(/([A-Z])/g, '-$1').toLowerCase());
                if (element) {
                    if (element.type === 'checkbox') {
                        element.checked = settings[key];
                    } else {
                        element.value = settings[key];
                    }
                }
            });
        } else {
            // 加载默认值 - 白名单和黑名单默认为空
            loadWhitelistItems([]);
            loadBlacklistItems([]);
        }
    }

    loadSettings();
}

/**
 * 初始化白名单黑名单管理
 */
function initWhitelistBlacklistManagement() {
    // 白名单管理
    const whitelistInput = document.getElementById('whitelist-input');
    const addWhitelistBtn = document.getElementById('add-whitelist-btn');
    
    if (whitelistInput && addWhitelistBtn) {
        addWhitelistBtn.addEventListener('click', () => {
            const item = whitelistInput.value.trim();
            if (item) {
                addWhitelistItem(item);
                whitelistInput.value = '';
            }
        });
        
        whitelistInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                const item = whitelistInput.value.trim();
                if (item) {
                    addWhitelistItem(item);
                    whitelistInput.value = '';
                }
            }
        });
    }
    
    // 黑名单管理
    const blacklistInput = document.getElementById('blacklist-input');
    const addBlacklistBtn = document.getElementById('add-blacklist-btn');
    
    if (blacklistInput && addBlacklistBtn) {
        addBlacklistBtn.addEventListener('click', () => {
            const item = blacklistInput.value.trim();
            if (item) {
                addBlacklistItem(item);
                blacklistInput.value = '';
            }
        });
        
        blacklistInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                const item = blacklistInput.value.trim();
                if (item) {
                    addBlacklistItem(item);
                    blacklistInput.value = '';
                }
            }
        });
    }
}

/**
 * 添加白名单项
 */
function addWhitelistItem(item) {
    if (!item || !item.trim()) return;
    
    // 检查是否已存在
    const existingItems = getWhitelistItems();
    if (existingItems.includes(item)) {
        showNotification('重复项目', '该应用已在白名单中');
        return;
    }
    
    const whitelistList = document.getElementById('whitelist-list');
    const whitelistEmpty = document.getElementById('whitelist-empty');
    
    if (!whitelistList) return;
    
    // 隐藏空状态提示
    if (whitelistEmpty) {
        whitelistEmpty.style.display = 'none';
    }
    
    // 创建列表项
    const listItem = document.createElement('div');
    listItem.className = 'flex items-center justify-between bg-gray-800 p-2 rounded-lg group hover:bg-gray-700 transition-colors';
    listItem.innerHTML = `
        <div class="flex items-center space-x-2">
            <i class="fas fa-check-circle text-green-400 text-sm"></i>
            <span class="text-gray-200 text-sm">${item}</span>
        </div>
        <button class="text-red-400 hover:text-red-300 opacity-0 group-hover:opacity-100 transition-opacity p-1" onclick="removeWhitelistItem(this, '${item}')">
            <i class="fas fa-trash text-sm"></i>
        </button>
    `;
    
    whitelistList.appendChild(listItem);
    console.log('白名单项已添加:', item);
    
    // 自动保存用户设置
    autoSaveUserSettings();
}

/**
 * 移除白名单项
 */
function removeWhitelistItem(button, item) {
    const listItem = button.closest('div.flex');
    if (listItem) {
        listItem.remove();
        console.log('白名单项已移除:', item);
        
        // 自动保存用户设置
        autoSaveUserSettings();
        
        // 检查是否需要显示空状态
        const whitelistList = document.getElementById('whitelist-list');
        const whitelistEmpty = document.getElementById('whitelist-empty');
        
        if (whitelistList && whitelistList.children.length === 0 && whitelistEmpty) {
            whitelistEmpty.style.display = 'block';
        }
    }
}

/**
 * 添加黑名单项
 */
function addBlacklistItem(item) {
    if (!item || !item.trim()) return;
    
    // 检查是否已存在
    const existingItems = getBlacklistItems();
    if (existingItems.includes(item)) {
        showNotification('重复项目', '该应用已在黑名单中');
        return;
    }
    
    const blacklistList = document.getElementById('blacklist-list');
    const blacklistEmpty = document.getElementById('blacklist-empty');
    
    if (!blacklistList) return;
    
    // 隐藏空状态提示
    if (blacklistEmpty) {
        blacklistEmpty.style.display = 'none';
    }
    
    // 创建列表项
    const listItem = document.createElement('div');
    listItem.className = 'flex items-center justify-between bg-gray-800 p-2 rounded-lg group hover:bg-gray-700 transition-colors';
    listItem.innerHTML = `
        <div class="flex items-center space-x-2">
            <i class="fas fa-times-circle text-red-400 text-sm"></i>
            <span class="text-gray-200 text-sm">${item}</span>
        </div>
        <button class="text-red-400 hover:text-red-300 opacity-0 group-hover:opacity-100 transition-opacity p-1" onclick="removeBlacklistItem(this, '${item}')">
            <i class="fas fa-trash text-sm"></i>
        </button>
    `;
    
    blacklistList.appendChild(listItem);
    console.log('黑名单项已添加:', item);
    
    // 自动保存用户设置
    autoSaveUserSettings();
}

/**
 * 移除黑名单项
 */
function removeBlacklistItem(button, item) {
    const listItem = button.closest('div.flex');
    if (listItem) {
        listItem.remove();
        console.log('黑名单项已移除:', item);
        
        // 自动保存用户设置
        autoSaveUserSettings();
        
        // 检查是否需要显示空状态
        const blacklistList = document.getElementById('blacklist-list');
        const blacklistEmpty = document.getElementById('blacklist-empty');
        
        if (blacklistList && blacklistList.children.length === 0 && blacklistEmpty) {
            blacklistEmpty.style.display = 'block';
        }
    }
}

/**
 * 获取白名单项目列表
 */
function getWhitelistItems() {
    const whitelistList = document.getElementById('whitelist-list');
    if (!whitelistList) return [];
    
    const items = [];
    const listItems = whitelistList.querySelectorAll('div.flex');
    listItems.forEach(item => {
        const span = item.querySelector('span');
        if (span) {
            items.push(span.textContent.trim());
        }
    });
    
    return items;
}

/**
 * 获取黑名单项目列表
 */
function getBlacklistItems() {
    const blacklistList = document.getElementById('blacklist-list');
    if (!blacklistList) return [];
    
    const items = [];
    const listItems = blacklistList.querySelectorAll('div.flex');
    listItems.forEach(item => {
        const span = item.querySelector('span');
        if (span) {
            items.push(span.textContent.trim());
        }
    });
    
    return items;
}

/**
 * 自动保存用户设置（无需通知）
 */
async function autoSaveUserSettings() {
    try {
        const settings = {
            theme: getCurrentTheme(),
            whitelist: getWhitelistItems(),
            blacklist: getBlacklistItems(),
            autostart: document.getElementById('autostart')?.checked || false,
            fatigue_notify: document.getElementById('fatigue-notify')?.checked || false,
            focus_duration: 25,
            short_break: 5,
            long_break: 15,
            
            // 分心干预设置
            distraction_intervention: {
                enabled: document.getElementById('distraction-intervention-enabled')?.checked || true,
                light_distraction_notification: document.getElementById('light-distraction-notification')?.checked || true,
                severe_distraction_popup: document.getElementById('severe-distraction-popup')?.checked || true,
                encouragement_enabled: document.getElementById('encouragement-enabled')?.checked || true,
                intervention_cooldown_minutes: parseInt(document.getElementById('intervention-cooldown')?.value || '5'),
                notification_sound: document.getElementById('intervention-sound')?.checked || true,
                popup_duration_seconds: parseInt(document.getElementById('popup-duration')?.value || '10'),
                encouragement_frequency: document.getElementById('encouragement-frequency')?.value || 'medium'
            }
        };

        await TauriAPI.saveUserSettings(settings);
        console.log('用户设置已自动保存:', settings);
        
        // 同时保存到本地存储作为备份
        localStorage.setItem('myFocusSettings', JSON.stringify(settings));
    } catch (error) {
        console.error('自动保存用户设置失败:', error);
    }
}

/**
 * 加载白名单项目
 */
function loadWhitelistItems(items) {
    if (!Array.isArray(items)) return;
    
    const whitelistList = document.getElementById('whitelist-list');
    const whitelistEmpty = document.getElementById('whitelist-empty');
    
    if (!whitelistList) return;
    
    // 清空现有项目
    whitelistList.innerHTML = '';
    
    if (items.length > 0) {
        // 隐藏空状态提示
        if (whitelistEmpty) {
            whitelistEmpty.style.display = 'none';
        }
        
        // 添加所有项目
        items.forEach(item => {
            if (item && item.trim()) {
                addWhitelistItem(item.trim());
            }
        });
    } else {
        // 显示空状态提示
        if (whitelistEmpty) {
            whitelistEmpty.style.display = 'block';
        }
    }
}

/**
 * 加载黑名单项目
 */
function loadBlacklistItems(items) {
    if (!Array.isArray(items)) return;
    
    const blacklistList = document.getElementById('blacklist-list');
    const blacklistEmpty = document.getElementById('blacklist-empty');
    
    if (!blacklistList) return;
    
    // 清空现有项目
    blacklistList.innerHTML = '';
    
    if (items.length > 0) {
        // 隐藏空状态提示
        if (blacklistEmpty) {
            blacklistEmpty.style.display = 'none';
        }
        
        // 添加所有项目
        items.forEach(item => {
            if (item && item.trim()) {
                addBlacklistItem(item.trim());
            }
        });
    } else {
        // 显示空状态提示
        if (blacklistEmpty) {
            blacklistEmpty.style.display = 'block';
        }
    }
}

/**
 * 初始化分心干预设置帮助提示
 */
function initInterventionTooltips() {
    const interventionHelp = document.getElementById('intervention-help');
    const interventionTooltip = document.getElementById('intervention-tooltip');
    
    if (interventionHelp && interventionTooltip) {
        let hoverTimeout;
        
        interventionHelp.addEventListener('mouseenter', () => {
            hoverTimeout = setTimeout(() => {
                interventionTooltip.classList.remove('hidden');
            }, 200);
        });
        
        interventionHelp.addEventListener('mouseleave', () => {
            clearTimeout(hoverTimeout);
            interventionTooltip.classList.add('hidden');
        });
        
        interventionHelp.addEventListener('click', (e) => {
            e.preventDefault();
            interventionTooltip.classList.toggle('hidden');
        });
        
        document.addEventListener('click', (e) => {
            if (!interventionHelp.contains(e.target) && !interventionTooltip.contains(e.target)) {
                interventionTooltip.classList.add('hidden');
            }
        });
    }
    
    // 初始化分心干预设置的自动保存事件
    initInterventionSettingsAutoSave();
}

/**
 * 初始化分心干预设置的自动保存
 */
function initInterventionSettingsAutoSave() {
    const interventionInputs = [
        'distraction-intervention-enabled',
        'light-distraction-notification', 
        'severe-distraction-popup',
        'encouragement-enabled',
        'intervention-cooldown',
        'intervention-sound',
        'popup-duration',
        'encouragement-frequency'
    ];
    
    interventionInputs.forEach(inputId => {
        const element = document.getElementById(inputId);
        if (element) {
            if (element.type === 'checkbox') {
                element.addEventListener('change', autoSaveUserSettings);
            } else if (element.tagName === 'SELECT') {
                element.addEventListener('change', autoSaveUserSettings);
            }
        }
    });
}

/**
 * 恢复默认分心干预设置
 */
function restoreDefaultInterventionSettings() {
    if (document.getElementById('distraction-intervention-enabled')) {
        document.getElementById('distraction-intervention-enabled').checked = true;
    }
    if (document.getElementById('light-distraction-notification')) {
        document.getElementById('light-distraction-notification').checked = true;
    }
    if (document.getElementById('severe-distraction-popup')) {
        document.getElementById('severe-distraction-popup').checked = true;
    }
    if (document.getElementById('encouragement-enabled')) {
        document.getElementById('encouragement-enabled').checked = true;
    }
    if (document.getElementById('intervention-cooldown')) {
        document.getElementById('intervention-cooldown').value = '5';
    }
    if (document.getElementById('intervention-sound')) {
        document.getElementById('intervention-sound').checked = true;
    }
    if (document.getElementById('popup-duration')) {
        document.getElementById('popup-duration').value = '10';
    }
    if (document.getElementById('encouragement-frequency')) {
        document.getElementById('encouragement-frequency').value = 'medium';
    }
}

function initHelpTooltips() {
    const monitoringHelp = document.getElementById('monitoring-help');
    const monitoringTooltip = document.getElementById('monitoring-tooltip');
    
    if (monitoringHelp && monitoringTooltip) {
        let hoverTimeout;
        
        monitoringHelp.addEventListener('mouseenter', () => {
            hoverTimeout = setTimeout(() => {
                monitoringTooltip.classList.remove('hidden');
            }, 200); // 200ms延迟显示
        });
        
        monitoringHelp.addEventListener('mouseleave', () => {
            clearTimeout(hoverTimeout);
            monitoringTooltip.classList.add('hidden');
        });
        
        // 点击也可以切换显示状态
        monitoringHelp.addEventListener('click', (e) => {
            e.preventDefault();
            monitoringTooltip.classList.toggle('hidden');
        });
        
        // 点击其他地方隐藏提示
        document.addEventListener('click', (e) => {
            if (!monitoringHelp.contains(e.target) && !monitoringTooltip.contains(e.target)) {
                monitoringTooltip.classList.add('hidden');
            }
        });
    }
    
    // AI设置帮助提示
    const aiHelp = document.getElementById('ai-help');
    const aiTooltip = document.getElementById('ai-tooltip');
    
    if (aiHelp && aiTooltip) {
        let hoverTimeout;
        
        aiHelp.addEventListener('mouseenter', () => {
            hoverTimeout = setTimeout(() => {
                aiTooltip.classList.remove('hidden');
            }, 200);
        });
        
        aiHelp.addEventListener('mouseleave', () => {
            clearTimeout(hoverTimeout);
            aiTooltip.classList.add('hidden');
        });
        
        aiHelp.addEventListener('click', (e) => {
            e.preventDefault();
            aiTooltip.classList.toggle('hidden');
        });
        
        document.addEventListener('click', (e) => {
            if (!aiHelp.contains(e.target) && !aiTooltip.contains(e.target)) {
                aiTooltip.classList.add('hidden');
            }
        });
    }
}

/**
 * 初始化AI模型设置
 */
function initAIModelSettings() {
    // 绑定测试API按钮
    const testApiBtn = document.getElementById('test-api-btn');
    if (testApiBtn) {
        testApiBtn.addEventListener('click', testAPIConnection);
    }
    
    // 绑定刷新模型按钮
    const refreshDetectionBtn = document.getElementById('refresh-detection-models');
    const refreshReportBtn = document.getElementById('refresh-report-models');
    
    if (refreshDetectionBtn) {
        refreshDetectionBtn.addEventListener('click', () => refreshModels('detection'));
    }
    
    if (refreshReportBtn) {
        refreshReportBtn.addEventListener('click', () => refreshModels('report'));
    }
    
    // 绑定API类型变化事件
    const apiTypeSelect = document.getElementById('api-type');
    if (apiTypeSelect) {
        apiTypeSelect.addEventListener('change', onAPITypeChanged);
    }
    
    // 移除自动保存，改为手动保存机制
    // 添加保存设置按钮事件处理器
    const saveSettingsBtn = document.getElementById('save-settings-btn');
    if (saveSettingsBtn) {
        saveSettingsBtn.addEventListener('click', handleSaveSettings);
    }
    
    // 加载AI配置
    loadAIConfig();
}

/**
 * API类型变化处理
 */
function onAPITypeChanged() {
    const apiTypeSelect = document.getElementById('api-type');
    const apiUrlInput = document.getElementById('api-url');
    
    if (!apiTypeSelect || !apiUrlInput) return;
    
    const selectedType = apiTypeSelect.value;
    const defaultUrls = getDefaultAPIUrls();
    
    if (defaultUrls[selectedType]) {
        apiUrlInput.value = defaultUrls[selectedType];
        
        // 清除之前的测试结果和模型列表
        clearAPITestResults();
        
        console.log(`API类型已切换到: ${selectedType}, URL已更新为: ${defaultUrls[selectedType]}`);
        showNotification('API类型已更新', `URL已自动设置为 ${selectedType} 的默认地址，请点击"保存设置"以保存配置`, 'info', false);
    }
}

/**
 * 获取不同API类型的默认URL配置
 */
function getDefaultAPIUrls() {
    return {
        'OpenAI Compatible': 'https://api.openai.com/v1',
        'Ollama (本地)': 'http://localhost:11434/v1',
        'Claude API': 'https://api.anthropic.com/v1'
    };
}

/**
 * 清除API测试结果和模型列表
 */
function clearAPITestResults() {
    // 清除测试结果
    const resultDiv = document.getElementById('api-test-result');
    if (resultDiv) {
        resultDiv.classList.add('hidden');
    }
    
    // 重置模型选择器
    const detectionSelect = document.getElementById('detection-model');
    const reportSelect = document.getElementById('report-model');
    
    if (detectionSelect) {
        detectionSelect.innerHTML = '<option value="">请先测试API连接</option>';
    }
    
    if (reportSelect) {
        reportSelect.innerHTML = '<option value="">请先测试API连接</option>';
    }
    
    // 重置模型状态显示
    const modelsLoading = document.getElementById('models-loading');
    const modelsCount = document.getElementById('models-count');
    
    if (modelsLoading) {
        modelsLoading.innerHTML = `
            <i class="fas fa-robot text-2xl mb-2 opacity-50"></i>
            <p>测试API连接以加载可用模型</p>
        `;
    }
    
    if (modelsCount) {
        modelsCount.textContent = '0 个模型';
    }
}

/**
 * 测试API连接
 */
async function testAPIConnection() {
    const testBtn = document.getElementById('test-api-btn');
    const resultDiv = document.getElementById('api-test-result');
    
    if (!testBtn || !resultDiv) return;
    
    // 获取当前配置
    const config = getCurrentAIConfig();
    
    if (!config.api_key.trim()) {
        showTestResult({
            success: false,
            message: 'API Key不能为空',
            response_time_ms: 0
        });
        return;
    }
    
    // 显示测试中状态
    testBtn.disabled = true;
    testBtn.innerHTML = '<i class="fas fa-spinner fa-spin"></i> <span>测试中...</span>';
    
    try {
        // 调用后端API测试
        const result = await TauriAPI.testAIAPI(config);
        showTestResult(result);
        
        // 如果测试成功，加载模型列表并自动保存配置
        if (result.success) {
            await loadAvailableModels();
            // 提示用户保存配置
            showNotification('API测试成功', '请点击"保存设置"按钮保存配置', 'success', false);
        }
        
    } catch (error) {
        console.error('API测试失败:', error);
        showTestResult({
            success: false,
            message: `测试失败: ${error}`,
            response_time_ms: 0
        });
    } finally {
        // 恢复按钮状态
        testBtn.disabled = false;
        testBtn.innerHTML = '<i class="fas fa-plug"></i> <span>测试连接</span>';
    }
}

/**
 * 显示API测试结果
 */
function showTestResult(result) {
    const resultDiv = document.getElementById('api-test-result');
    if (!resultDiv) return;
    
    resultDiv.classList.remove('hidden');
    
    const statusClass = result.success ? 'text-green-400' : 'text-red-400';
    const statusIcon = result.success ? 'fa-check-circle' : 'fa-times-circle';
    
    resultDiv.innerHTML = `
        <div class="flex items-center space-x-2 p-3 rounded-lg ${result.success ? 'bg-green-900/20 border border-green-500/30' : 'bg-red-900/20 border border-red-500/30'}">
            <i class="fas ${statusIcon} ${statusClass}"></i>
            <div class="flex-grow">
                <p class="${statusClass} font-medium">${result.message}</p>
                ${result.response_time_ms > 0 ? `<p class="text-xs text-gray-400 mt-1">响应时间: ${result.response_time_ms}ms</p>` : ''}
            </div>
        </div>
    `;
}

/**
 * 加载可用模型列表
 */
async function loadAvailableModels() {
    const config = getCurrentAIConfig();
    const modelsStatus = document.getElementById('models-status');
    const modelsLoading = document.getElementById('models-loading');
    const modelsCount = document.getElementById('models-count');
    
    if (!modelsStatus || !modelsLoading || !modelsCount) return;
    
    // 显示加载状态
    modelsLoading.innerHTML = `
        <i class="fas fa-spinner fa-spin text-2xl mb-2 opacity-50"></i>
        <p>正在加载模型列表...</p>
    `;
    
    try {
        const models = await TauriAPI.getAvailableModels(config);
        console.log('获取到模型列表:', models);
        
        // 更新模型下拉框
        updateModelSelectors(models);
        
        // 更新状态显示
        modelsCount.textContent = `${models.length} 个模型`;
        
        if (models.length > 0) {
            modelsLoading.innerHTML = `
                <div class="text-center text-green-400 py-2">
                    <i class="fas fa-check-circle text-xl mb-1"></i>
                    <p class="text-sm">成功加载 ${models.length} 个可用模型</p>
                </div>
            `;
        } else {
            modelsLoading.innerHTML = `
                <div class="text-center text-yellow-400 py-2">
                    <i class="fas fa-exclamation-triangle text-xl mb-1"></i>
                    <p class="text-sm">未找到可用模型</p>
                </div>
            `;
        }
        
    } catch (error) {
        console.error('加载模型列表失败:', error);
        modelsLoading.innerHTML = `
            <div class="text-center text-red-400 py-2">
                <i class="fas fa-times-circle text-xl mb-1"></i>
                <p class="text-sm">加载模型列表失败</p>
            </div>
        `;
    }
}

/**
 * 更新模型选择器
 */
function updateModelSelectors(models) {
    const detectionSelect = document.getElementById('detection-model');
    const reportSelect = document.getElementById('report-model');
    
    if (!detectionSelect || !reportSelect) return;
    
    // 保存当前选择的值
    const currentDetection = detectionSelect.value;
    const currentReport = reportSelect.value;
    
    // 清空并重新填充选项
    [detectionSelect, reportSelect].forEach(select => {
        select.innerHTML = '<option value="">请选择模型</option>';
        
        models.forEach(model => {
            const option = document.createElement('option');
            option.value = model.id;
            option.textContent = model.id;
            select.appendChild(option);
        });
    });
    
    // 恢复之前的选择（如果仍然存在）
    if (models.some(m => m.id === currentDetection)) {
        detectionSelect.value = currentDetection;
    }
    
    if (models.some(m => m.id === currentReport)) {
        reportSelect.value = currentReport;
    }
}

/**
 * 刷新模型列表
 */
async function refreshModels(type) {
    console.log(`刷新${type}模型列表`);
    
    const config = getCurrentAIConfig();
    if (!config.api_key.trim()) {
        showNotification('配置错误', '请先配置并测试API连接');
        return;
    }
    
    await loadAvailableModels();
    showNotification('刷新成功', '模型列表已更新');
}

/**
 * 获取当前AI配置
 */
function getCurrentAIConfig() {
    return {
        api_type: document.getElementById('api-type')?.value || 'OpenAI Compatible',
        api_url: document.getElementById('api-url')?.value || 'https://api.openai.com/v1',
        api_key: document.getElementById('api-key')?.value || '',
        detection_model: document.getElementById('detection-model')?.value || '',
        report_model: document.getElementById('report-model')?.value || ''
    };
}

/**
 * 自动保存AI配置（无需通知）
 */
async function autoSaveAIConfig() {
    const config = getCurrentAIConfig();
    
    try {
        await TauriAPI.saveAIConfig(config);
        console.log('AI配置已自动保存:', config);
    } catch (error) {
        console.error('自动保存AI配置失败:', error);
    }
}

/**
 * 保存AI配置
 */
async function saveAIConfig() {
    const config = getCurrentAIConfig();
    
    try {
        await TauriAPI.saveAIConfig(config);
        console.log('AI配置已保存:', config);
    } catch (error) {
        console.error('保存AI配置失败:', error);
        showNotification('保存失败', 'AI配置保存失败');
    }
}

/**
 * 加载AI配置
 */
async function loadAIConfig() {
    try {
        const config = await TauriAPI.loadAIConfig();
        console.log('加载AI配置:', config);
        
        // 恢复配置到界面
        if (document.getElementById('api-type')) {
            document.getElementById('api-type').value = config.api_type || 'OpenAI Compatible';
        }
        if (document.getElementById('api-url')) {
            document.getElementById('api-url').value = config.api_url || 'https://api.openai.com/v1';
        }
        if (document.getElementById('api-key')) {
            document.getElementById('api-key').value = config.api_key || '';
        }
        if (document.getElementById('detection-model')) {
            document.getElementById('detection-model').value = config.detection_model || '';
        }
        if (document.getElementById('report-model')) {
            document.getElementById('report-model').value = config.report_model || '';
        }
        
        // 如果URL为空，根据API类型设置默认URL
        const apiUrl = config.api_url;
        const apiType = config.api_type;
        const defaultUrls = getDefaultAPIUrls();
        
        if (!apiUrl && apiType && defaultUrls[apiType]) {
            document.getElementById('api-url').value = defaultUrls[apiType];
            console.log(`URL已自动设置为${apiType}的默认地址: ${defaultUrls[apiType]}`);
        }
        
    } catch (error) {
        console.error('加载AI配置失败:', error);
    }
}

/**
 * 更新仪表盘数据
 */
async function updateDashboard() {
    console.log('📊 更新仪表盘数据...');
    
    try {
        // 调用现有的真实数据更新函数
        await updateDashboardWithRealData();
        
        // 更新计时器显示
        await updateTimerDisplay();
        
        // 更新疲劳度
        await updateFatigueLevel();
        
        console.log('✅ 仪表盘更新完成');
        
    } catch (error) {
        console.error('❌ 仪表盘更新失败:', error);
        showNotification(`仪表盘更新失败: ${error.message}`, 'error');
    }
}

/**
 * 使用真实数据更新仪表盘
 */
async function updateDashboardWithRealData() {
    try {
        // 获取今日统计数据
        const stats = await TauriAPI.getTodayStatistics();
        console.log('今日统计数据:', stats);
        
        // 更新页面显示
        updateStatsDisplay(stats);
        
        // 加载任务列表
        const tasks = await TauriAPI.getTasks();
        console.log('任务列表:', tasks);
        updateTasksList(tasks);
        
    } catch (error) {
        console.error('仪表盘数据更新失败:', error);
    }
}

/**
 * 更新统计数据显示
 */
function updateStatsDisplay(stats) {
    if (!stats) {
        // 如果没有统计数据，显示默认值
        updateStatsElements(0, 0, 0);
        return;
    }
    
    try {
        // 处理不同格式的统计数据
        let focusTime = 0;
        let distractTime = 0;
        let focusScore = 0;

        if (typeof stats === 'string') {
            // 如果返回的是字符串，显示默认值
            updateStatsElements(0, 0, 0);
            return;
        }

        if (typeof stats === 'object') {
            focusTime = stats.total_focus_time || 0;
            distractTime = stats.total_distract_time || 0;  
            focusScore = stats.focus_score || 0;
        }

        updateStatsElements(focusTime, distractTime, focusScore);
        
    } catch (error) {
        console.error('更新统计数据显示失败:', error);
        updateStatsElements(0, 0, 0);
    }
}

/**
 * 更新统计数据元素
 */
function updateStatsElements(focusTimeSeconds, distractTimeSeconds, focusScore) {
    // 更新专注时间显示
    const focusTimeElement = document.getElementById('today-focus-time');
    if (focusTimeElement) {
        const focusHours = Math.floor(focusTimeSeconds / 3600);
        const focusMinutes = Math.floor((focusTimeSeconds % 3600) / 60);
        focusTimeElement.textContent = `${focusHours}h ${focusMinutes}m`;
    }

    // 更新分心时间显示
    const distractTimeElement = document.getElementById('today-distract-time');
    if (distractTimeElement) {
        const distractMinutes = Math.floor(distractTimeSeconds / 60);
        distractTimeElement.textContent = `${distractMinutes}m`;
    }

    // 更新进度条
    const totalTime = focusTimeSeconds + distractTimeSeconds;
    if (totalTime > 0) {
        const focusPercentage = (focusTimeSeconds / totalTime) * 100;
        const distractPercentage = (distractTimeSeconds / totalTime) * 100;

        const focusProgressBar = document.getElementById('focus-progress-bar');
        if (focusProgressBar) {
            focusProgressBar.style.width = `${focusPercentage}%`;
        }

        const distractProgressBar = document.getElementById('distract-progress-bar');
        if (distractProgressBar) {
            distractProgressBar.style.width = `${distractPercentage}%`;
        }
    } else {
        // 如果没有数据，进度条为0
        const focusProgressBar = document.getElementById('focus-progress-bar');
        if (focusProgressBar) {
            focusProgressBar.style.width = '0%';
        }

        const distractProgressBar = document.getElementById('distract-progress-bar');
        if (distractProgressBar) {
            distractProgressBar.style.width = '0%';
        }
    }
}

/**
 * 更新任务列表显示
 */
function updateTasksList(tasks) {
    const taskList = document.getElementById('task-list');
    if (!taskList || !Array.isArray(tasks)) return;
    
    // 清空现有任务（除了空任务提示）
    const existingTasks = taskList.querySelectorAll('[data-task-id]');
    existingTasks.forEach(task => task.remove());
    
    // 如果有任务，隐藏空任务提示并添加任务
    if (tasks.length > 0) {
        hideEmptyTaskMessage();
        tasks.forEach(task => {
            const taskItem = createTaskElement(task);
            taskList.appendChild(taskItem);
        });
    } else {
        // 如果没有任务，显示空任务提示
        showEmptyTaskMessage();
    }
    
    // 同步任务到计时器界面
    updateTaskSelector();
}

/**
 * 更新计时器显示
 */
async function updateTimerDisplay() {
    console.log('⏱️ 更新计时器显示...');
    
    try {
        // 获取计时器状态
        const timerStatus = await TauriAPI.getTimerStatus();
        if (!timerStatus) {
            console.warn('无法获取计时器状态');
            return;
        }
        
        // 更新计时器时间显示
        const timerTimeElement = document.getElementById('timer-time');
        const timerProgressElement = document.getElementById('timer-progress');
        const timerStatusElement = document.getElementById('timer-status');
        const timerControlsElement = document.getElementById('timer-controls');
        
        if (timerStatus.is_running) {
            // 计时器运行中
            const remainingMinutes = Math.floor(timerStatus.remaining_seconds / 60);
            const remainingSeconds = timerStatus.remaining_seconds % 60;
            const elapsedMinutes = Math.floor(timerStatus.elapsed_seconds / 60);
            const totalMinutes = timerStatus.duration_minutes;
            
            // 更新时间显示
            if (timerTimeElement) {
                timerTimeElement.textContent = `${String(remainingMinutes).padStart(2, '0')}:${String(remainingSeconds).padStart(2, '0')}`;
            }
            
            // 更新进度条
            if (timerProgressElement && totalMinutes > 0) {
                const progressPercent = (elapsedMinutes / totalMinutes) * 100;
                timerProgressElement.style.width = `${progressPercent}%`;
                timerProgressElement.className = 'h-full bg-blue-500 rounded-full transition-all duration-1000';
            }
            
            // 更新状态显示
            if (timerStatusElement) {
                const sessionTypeText = timerStatus.session_type === 'Focus' ? '专注' : 
                                      timerStatus.session_type === 'ShortBreak' ? '短休息' : 
                                      timerStatus.session_type === 'LongBreak' ? '长休息' : '计时';
                timerStatusElement.textContent = `${sessionTypeText}中 (${elapsedMinutes}/${totalMinutes}分钟)`;
                timerStatusElement.className = 'text-sm text-blue-400';
            }
            
            // 更新控制按钮
            if (timerControlsElement) {
                timerControlsElement.innerHTML = `
                    <button onclick="pauseTimer()" class="px-4 py-2 bg-yellow-600 text-white rounded hover:bg-yellow-700">
                        <i class="fas fa-pause mr-1"></i>暂停
                    </button>
                    <button onclick="stopTimer()" class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700">
                        <i class="fas fa-stop mr-1"></i>停止
                    </button>
                `;
            }
            
        } else {
            // 计时器未运行
            if (timerTimeElement) {
                timerTimeElement.textContent = '00:00';
            }
            
            if (timerProgressElement) {
                timerProgressElement.style.width = '0%';
                timerProgressElement.className = 'h-full bg-gray-500 rounded-full transition-all duration-500';
            }
            
            if (timerStatusElement) {
                timerStatusElement.textContent = '计时器已停止';
                timerStatusElement.className = 'text-sm text-gray-400';
            }
            
            if (timerControlsElement) {
                timerControlsElement.innerHTML = `
                    <button onclick="startFocusSession()" class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700">
                        <i class="fas fa-play mr-1"></i>开始专注
                    </button>
                `;
            }
        }
        
        // 更新任务关联显示
        const taskInfoElement = document.getElementById('timer-task-info');
        if (taskInfoElement) {
            if (timerStatus.task_id && currentSelectedTask) {
                taskInfoElement.innerHTML = `
                    <div class="text-sm text-gray-400">
                        <i class="fas fa-tasks mr-1"></i>关联任务: ${currentSelectedTask.text}
                    </div>
                `;
            } else {
                taskInfoElement.innerHTML = '';
            }
        }
        
        console.log('✅ 计时器显示更新完成');
        
    } catch (error) {
        console.error('❌ 更新计时器显示失败:', error);
        
        // 显示错误状态
        const timerTimeElement = document.getElementById('timer-time');
        const timerStatusElement = document.getElementById('timer-status');
        
        if (timerTimeElement) {
            timerTimeElement.textContent = '--:--';
        }
        if (timerStatusElement) {
            timerStatusElement.textContent = '无法获取计时器状态';
            timerStatusElement.className = 'text-sm text-red-400';
        }
    }
}

/**
 * 加载报告数据
 */
async function loadReports() {
    console.log('📊 开始加载报告数据...');
    
    try {
        // 获取报告列表容器
        const reportsList = document.getElementById('reports-list');
        if (!reportsList) {
            console.warn('未找到报告列表容器');
            return;
        }

        // 显示加载状态
        reportsList.innerHTML = `
            <div class="flex items-center justify-center py-8">
                <div class="text-center">
                    <i class="fas fa-spinner fa-spin text-2xl text-blue-400 mb-2"></i>
                    <p class="text-gray-400">正在加载报告...</p>
                </div>
            </div>
        `;

        // 获取日报告和周报告列表
        const [dailyReports, weeklyReports] = await Promise.all([
            TauriAPI.getReportList('daily', 10),
            TauriAPI.getReportList('weekly', 4)
        ]);

        // 清空容器
        reportsList.innerHTML = '';

        // 添加日报告部分
        if (dailyReports && dailyReports.length > 0) {
            const dailySection = document.createElement('div');
            dailySection.className = 'mb-8';
            dailySection.innerHTML = `
                <h3 class="text-lg font-semibold text-white mb-4">
                    <i class="fas fa-calendar-day mr-2 text-blue-400"></i>日报告
                </h3>
                <div class="grid gap-4" id="daily-reports-grid"></div>
            `;
            reportsList.appendChild(dailySection);

            const dailyGrid = document.getElementById('daily-reports-grid');
            dailyReports.forEach(report => {
                const reportCard = createReportCard(report, 'daily');
                dailyGrid.appendChild(reportCard);
            });
        }

        // 添加周报告部分
        if (weeklyReports && weeklyReports.length > 0) {
            const weeklySection = document.createElement('div');
            weeklySection.innerHTML = `
                <h3 class="text-lg font-semibold text-white mb-4">
                    <i class="fas fa-calendar-week mr-2 text-green-400"></i>周报告
                </h3>
                <div class="grid gap-4" id="weekly-reports-grid"></div>
            `;
            reportsList.appendChild(weeklySection);

            const weeklyGrid = document.getElementById('weekly-reports-grid');
            weeklyReports.forEach(report => {
                const reportCard = createReportCard(report, 'weekly');
                weeklyGrid.appendChild(reportCard);
            });
        }

        // 如果没有报告
        if ((!dailyReports || dailyReports.length === 0) && 
            (!weeklyReports || weeklyReports.length === 0)) {
            reportsList.innerHTML = `
                <div class="text-center py-8">
                    <i class="fas fa-file-alt text-4xl text-gray-600 mb-4"></i>
                    <p class="text-gray-400 mb-4">暂无报告数据</p>
                    <p class="text-sm text-gray-500">开始监控后将自动生成专注报告</p>
                </div>
            `;
        }

        console.log('✅ 报告数据加载完成');
    } catch (error) {
        console.error('❌ 加载报告数据失败:', error);
        
        const reportsList = document.getElementById('reports-list');
        if (reportsList) {
            reportsList.innerHTML = `
                <div class="text-center py-8">
                    <i class="fas fa-exclamation-triangle text-4xl text-red-400 mb-4"></i>
                    <p class="text-red-400 mb-2">加载报告失败</p>
                    <p class="text-sm text-gray-500">${error.message}</p>
                    <button onclick="loadReports()" class="mt-4 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700">
                        重试
                    </button>
                </div>
            `;
        }
    }
}

/**
 * 创建报告卡片
 */
function createReportCard(report, type) {
    const card = document.createElement('div');
    card.className = 'bg-gray-800/60 p-4 rounded-lg hover:bg-gray-700/60 transition-colors cursor-pointer';
    
    const typeIcon = type === 'daily' ? 'fas fa-calendar-day' : 'fas fa-calendar-week';
    const typeColor = type === 'daily' ? 'text-blue-400' : 'text-green-400';
    
    card.innerHTML = `
        <div class="flex items-center justify-between mb-2">
            <div class="flex items-center">
                <i class="${typeIcon} ${typeColor} mr-2"></i>
                <span class="text-white font-medium">${report.title}</span>
            </div>
            <span class="text-xs text-gray-400">${report.status}</span>
        </div>
        <div class="text-sm text-gray-400 mb-3">
            ${formatDate(report.date)}
        </div>
        <div class="flex gap-2">
            <button onclick="viewReport('${report.id}', '${type}')" 
                    class="flex-1 px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700">
                <i class="fas fa-eye mr-1"></i>查看
            </button>
            <button onclick="downloadReport('${report.id}', '${type}')" 
                    class="px-3 py-1 bg-gray-600 text-white text-sm rounded hover:bg-gray-700">
                <i class="fas fa-download mr-1"></i>下载
            </button>
        </div>
    `;
    
    return card;
}

/**
 * 查看报告详情
 */
async function viewReport(reportId, type) {
    console.log(`📖 查看报告: ${reportId} (${type})`);
    
    try {
        // 显示加载提示
        showNotification('正在生成报告...', 'info');
        
        // 从报告ID中提取日期
        const dateMatch = reportId.match(/\d{4}-\d{2}-\d{2}/);
        if (!dateMatch) {
            throw new Error('无效的报告ID');
        }
        const date = dateMatch[0];
        
        let reportData;
        if (type === 'daily') {
            reportData = await TauriAPI.generateDailyReport(date);
        } else if (type === 'weekly') {
            reportData = await TauriAPI.generateWeeklyReport(date);
        } else {
            throw new Error('不支持的报告类型');
        }
        
        // 显示报告详情模态框
        showReportModal(reportData, type);
        
    } catch (error) {
        console.error('查看报告失败:', error);
        showNotification(`查看报告失败: ${error.message}`, 'error');
    }
}

/**
 * 下载报告
 */
async function downloadReport(reportId, type) {
    console.log(`💾 下载报告: ${reportId} (${type})`);
    
    try {
        showNotification('正在准备下载...', 'info');
        
        const dateMatch = reportId.match(/\d{4}-\d{2}-\d{2}/);
        if (!dateMatch) {
            throw new Error('无效的报告ID');
        }
        const date = dateMatch[0];
        
        // 生成报告数据
        let reportData;
        if (type === 'daily') {
            reportData = await TauriAPI.generateDailyReport(date);
        } else if (type === 'weekly') {
            reportData = await TauriAPI.generateWeeklyReport(date);
        }
        
        // 创建下载内容
        const content = JSON.stringify(reportData, null, 2);
        const blob = new Blob([content], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        
        // 创建下载链接
        const a = document.createElement('a');
        a.href = url;
        a.download = `${reportId}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        
        showNotification('报告下载成功', 'success');
        
    } catch (error) {
        console.error('下载报告失败:', error);
        showNotification(`下载失败: ${error.message}`, 'error');
    }
}

/**
 * 显示报告详情模态框
 */
function showReportModal(reportData, type) {
    const modal = document.createElement('div');
    modal.className = 'fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4';
    modal.onclick = (e) => {
        if (e.target === modal) {
            document.body.removeChild(modal);
        }
    };
    
    let content = '';
    
    if (type === 'daily') {
        content = `
            <h2 class="text-xl font-bold text-white mb-4">${reportData.date} 日报告</h2>
            
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
                <div class="bg-gray-700 p-3 rounded">
                    <p class="text-sm text-gray-400">专注时长</p>
                    <p class="text-lg font-bold text-green-400">${Math.floor(reportData.summary.focus_time_seconds / 60)}分钟</p>
                </div>
                <div class="bg-gray-700 p-3 rounded">
                    <p class="text-sm text-gray-400">专注分数</p>
                    <p class="text-lg font-bold text-blue-400">${reportData.summary.focus_score.toFixed(1)}%</p>
                </div>
                <div class="bg-gray-700 p-3 rounded">
                    <p class="text-sm text-gray-400">中断次数</p>
                    <p class="text-lg font-bold text-yellow-400">${reportData.summary.interruption_count}次</p>
                </div>
            </div>
            
            <div class="space-y-4">
                <div>
                    <h3 class="text-lg font-semibold text-white mb-2">AI洞察</h3>
                    <div class="bg-gray-700 p-3 rounded text-sm text-gray-300">
                        <p><strong>表现总结：</strong>${reportData.ai_insights.performance_summary}</p>
                        <p class="mt-2"><strong>改进建议：</strong>${reportData.ai_insights.productivity_suggestions}</p>
                    </div>
                </div>
                
                <div>
                    <h3 class="text-lg font-semibold text-white mb-2">应用使用情况</h3>
                    <div class="space-y-2">
                        ${reportData.app_usage.slice(0, 5).map(app => `
                            <div class="flex justify-between items-center bg-gray-700 p-2 rounded">
                                <span class="text-white">${app.app_name}</span>
                                <span class="text-gray-400">${Math.floor(app.total_time_seconds / 60)}分钟</span>
                            </div>
                        `).join('')}
                    </div>
                </div>
            </div>
        `;
    } else if (type === 'weekly') {
        content = `
            <h2 class="text-xl font-bold text-white mb-4">周报告 (${reportData.week_start} 至 ${reportData.week_end})</h2>
            
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
                <div class="bg-gray-700 p-3 rounded">
                    <p class="text-sm text-gray-400">总专注时长</p>
                    <p class="text-lg font-bold text-green-400">${Math.floor(reportData.summary.total_focus_time_seconds / 3600)}小时</p>
                </div>
                <div class="bg-gray-700 p-3 rounded">
                    <p class="text-sm text-gray-400">平均专注分数</p>
                    <p class="text-lg font-bold text-blue-400">${reportData.summary.average_daily_focus_score.toFixed(1)}%</p>
                </div>
                <div class="bg-gray-700 p-3 rounded">
                    <p class="text-sm text-gray-400">专注趋势</p>
                    <p class="text-lg font-bold text-purple-400">${reportData.summary.productivity_trend}</p>
                </div>
            </div>
            
            <div class="space-y-4">
                <div>
                    <h3 class="text-lg font-semibold text-white mb-2">每日趋势</h3>
                    <div class="space-y-2">
                        ${reportData.daily_trends.map(day => `
                            <div class="flex justify-between items-center bg-gray-700 p-2 rounded">
                                <span class="text-white">${day.date}</span>
                                <div class="flex gap-4 text-sm">
                                    <span class="text-green-400">${day.focus_score.toFixed(1)}%</span>
                                    <span class="text-gray-400">${Math.floor(day.focus_time_seconds / 60)}分钟</span>
                                </div>
                            </div>
                        `).join('')}
                    </div>
                </div>
                
                <div>
                    <h3 class="text-lg font-semibold text-white mb-2">AI洞察</h3>
                    <div class="bg-gray-700 p-3 rounded text-sm text-gray-300">
                        <p><strong>表现总结：</strong>${reportData.ai_insights.performance_summary}</p>
                        <p class="mt-2"><strong>改进建议：</strong>${reportData.ai_insights.productivity_suggestions}</p>
                    </div>
                </div>
            </div>
        `;
    }
    
    modal.innerHTML = `
        <div class="bg-gray-800 rounded-lg p-6 max-w-4xl max-h-[80vh] overflow-y-auto w-full">
            ${content}
            <div class="flex justify-end mt-6">
                <button onclick="this.closest('.fixed').remove()" 
                        class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700">
                    关闭
                </button>
            </div>
        </div>
    `;
    
    document.body.appendChild(modal);
}

/**
 * 格式化日期显示
 */
function formatDate(dateStr) {
    const date = new Date(dateStr);
    const now = new Date();
    const diffTime = now - date;
    const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));
    
    if (diffDays === 0) {
        return '今天';
    } else if (diffDays === 1) {
        return '昨天';
    } else if (diffDays < 7) {
        return `${diffDays}天前`;
    } else {
        return date.toLocaleDateString('zh-CN');
    }
}

/**
 * 更新疲劳度显示
 */
async function updateFatigueLevel() {
    console.log('🧠 更新疲劳度显示...');
    
    try {
        // 获取今日统计数据来计算疲劳度
        const stats = await TauriAPI.getTodayStatistics();
        if (!stats) {
            console.warn('无法获取统计数据');
            return;
        }
        
        // 计算疲劳度 (基于专注时间、中断次数等)
        const totalMinutes = Math.floor(stats.total_focus_time / 60);
        const interruptionCount = stats.interruption_count || 0;
        
        // 疲劳度计算逻辑：
        // - 基础疲劳度：专注时间越长，疲劳度越高
        // - 中断惩罚：每次中断增加疲劳度
        // - 专注分数影响：专注分数越低，疲劳度越高
        let fatigueLevel = Math.min(totalMinutes * 1.5, 60); // 基础疲劳度，最高60
        fatigueLevel += interruptionCount * 3; // 每次中断增加3点疲劳
        fatigueLevel += Math.max(0, (60 - stats.focus_score) * 0.5); // 专注分数低时增加疲劳
        
        // 限制在0-100范围内
        fatigueLevel = Math.max(0, Math.min(100, Math.round(fatigueLevel)));
        
        // 更新疲劳度显示
        const fatigueElement = document.getElementById('fatigue-level');
        const fatigueTextElement = document.getElementById('fatigue-text');
        const fatigueProgressElement = document.getElementById('fatigue-progress');
        
        if (fatigueElement) {
            fatigueElement.textContent = fatigueLevel;
        }
        
        // 根据疲劳度确定状态文本和颜色
        let fatigueStatus, fatigueColor, progressColor;
        if (fatigueLevel < 30) {
            fatigueStatus = '精力充沛';
            fatigueColor = 'text-green-400';
            progressColor = 'bg-green-500';
        } else if (fatigueLevel < 60) {
            fatigueStatus = '轻度疲劳';
            fatigueColor = 'text-yellow-400';
            progressColor = 'bg-yellow-500';
        } else if (fatigueLevel < 80) {
            fatigueStatus = '中度疲劳';
            fatigueColor = 'text-orange-400';
            progressColor = 'bg-orange-500';
        } else {
            fatigueStatus = '重度疲劳';
            fatigueColor = 'text-red-400';
            progressColor = 'bg-red-500';
        }
        
        if (fatigueTextElement) {
            fatigueTextElement.textContent = fatigueStatus;
            fatigueTextElement.className = `text-sm ${fatigueColor}`;
        }
        
        // 更新进度条
        if (fatigueProgressElement) {
            fatigueProgressElement.style.width = `${fatigueLevel}%`;
            fatigueProgressElement.className = `h-2 rounded transition-all duration-500 ${progressColor}`;
        }
        
        // 更新疲劳度建议
        const suggestionElement = document.getElementById('fatigue-suggestion');
        if (suggestionElement) {
            let suggestion = '';
            if (fatigueLevel < 30) {
                suggestion = '状态良好，可以继续专注工作。';
            } else if (fatigueLevel < 60) {
                suggestion = '建议在30分钟后休息5-10分钟。';
            } else if (fatigueLevel < 80) {
                suggestion = '建议立即休息15分钟，进行放松活动。';
            } else {
                suggestion = '疲劳度过高，建议停止工作并充分休息。';
            }
            suggestionElement.textContent = suggestion;
        }
        
        console.log(`✅ 疲劳度更新完成: ${fatigueLevel} (${fatigueStatus})`);
        
    } catch (error) {
        console.error('❌ 更新疲劳度失败:', error);
        
        // 显示错误状态
        const fatigueElement = document.getElementById('fatigue-level');
        const fatigueTextElement = document.getElementById('fatigue-text');
        
        if (fatigueElement) {
            fatigueElement.textContent = '--';
        }
        if (fatigueTextElement) {
            fatigueTextElement.textContent = '无法获取';
            fatigueTextElement.className = 'text-sm text-gray-500';
        }
    }
}

/**
 * 显示通知
 */
function showNotification(title, message, type = 'info', showSystemNotification = false) {
    // 页面内通知（总是显示）
    showInPageNotification(title, message, type);
    
    // 系统通知（可选，默认关闭以避免重复）
    if (showSystemNotification && 'Notification' in window) {
        if (Notification.permission === 'granted') {
            const options = {
                body: message,
                icon: getNotificationIcon(type),
                tag: 'my-focus-notification'
            };
            new Notification(title, options);
        } else if (Notification.permission !== 'denied') {
            Notification.requestPermission().then(permission => {
                if (permission === 'granted') {
                    const options = {
                        body: message,
                        icon: getNotificationIcon(type),
                        tag: 'my-focus-notification'
                    };
                    new Notification(title, options);
                }
            });
        }
    }
}

/**
 * 显示页面内通知
 */
function showInPageNotification(title, message, type = 'info') {
    // 创建通知元素
    const notification = document.createElement('div');
    const colors = {
        'info': 'bg-blue-600 border-blue-500',
        'success': 'bg-green-600 border-green-500', 
        'warning': 'bg-yellow-600 border-yellow-500',
        'error': 'bg-red-600 border-red-500'
    };
    
    const icons = {
        'info': 'fa-info-circle',
        'success': 'fa-check-circle',
        'warning': 'fa-exclamation-triangle', 
        'error': 'fa-times-circle'
    };

    notification.className = `fixed top-4 right-4 ${colors[type] || colors.info} border-l-4 text-white p-4 rounded-lg shadow-lg z-50 max-w-sm transform translate-x-full transition-transform duration-300`;
    
    notification.innerHTML = `
        <div class="flex items-start">
            <i class="fas ${icons[type] || icons.info} text-xl mr-3 mt-0.5"></i>
            <div class="flex-1">
                <h4 class="font-semibold text-sm">${title}</h4>
                <p class="text-sm mt-1 opacity-90">${message.replace(/\n/g, '<br>')}</p>
            </div>
            <button onclick="this.parentElement.parentElement.remove()" class="ml-2 text-white/70 hover:text-white">
                <i class="fas fa-times"></i>
            </button>
        </div>
    `;

    document.body.appendChild(notification);

    // 动画显示
    setTimeout(() => {
        notification.classList.remove('translate-x-full');
    }, 100);

    // 自动消失
    setTimeout(() => {
        notification.classList.add('translate-x-full');
        setTimeout(() => {
            if (notification.parentNode) {
                notification.remove();
            }
        }, 300);
    }, type === 'error' ? 8000 : type === 'warning' ? 6000 : 4000);
}

/**
 * 获取通知图标
 */
function getNotificationIcon(type) {
    // 可以根据类型返回不同的图标路径
    return '/icon.ico'; // 默认应用图标
}

/**
 * 格式化时间
 */
function formatTime(seconds) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    if (hours > 0) {
        return `${hours}h ${minutes}m`;
    }
    return `${minutes}m ${secs}s`;
}

/**
 * 获取今日日期字符串
 */
function getTodayString() {
    const today = new Date();
    return today.toISOString().split('T')[0];
}

/**
 * 初始化数据图表（如果需要）
 */
function initCharts() {
    // 这里可以初始化各种数据图表
    console.log('Initializing charts...');
}

// 创建防抖版本的openTaskSelector
const debouncedOpenTaskSelector = debounce(openTaskSelector, 300);

/**
 * 初始化仪表盘控制功能
 */
function initDashboardControls() {
    // 初始化监控状态
    updateMonitoringButton();
    updateFocusStatus('idle');
    updateCurrentTaskDisplay();
    
    // 加载保存的当前任务
    loadCurrentTask();
    
    // 确保任务选择按钮事件绑定
    const selectTaskBtn = document.getElementById('select-task-btn');
    if (selectTaskBtn) {
        // 移除可能存在的旧事件监听器
        selectTaskBtn.removeEventListener('click', debouncedOpenTaskSelector);
        // 重新绑定事件（使用防抖版本）
        selectTaskBtn.addEventListener('click', (e) => {
            e.preventDefault();
            e.stopPropagation();
            console.log('选择任务按钮被点击');
            debouncedOpenTaskSelector();
        });
        console.log('任务选择按钮事件已绑定（防抖版本）');
    } else {
        console.warn('找不到任务选择按钮');
    }
    
    // 确保模态框按钮事件绑定
    setTimeout(() => {
        const modal = document.getElementById('task-selection-modal');
        if (modal) {
            // 为模态框内的按钮绑定事件
            const cancelBtn = modal.querySelector('button[onclick="closeTaskSelector()"]');
            const clearBtn = modal.querySelector('button[onclick="clearCurrentTask()"]');
            
            if (cancelBtn) {
                cancelBtn.removeAttribute('onclick');
                cancelBtn.addEventListener('click', closeTaskSelector);
            }
            
            if (clearBtn) {
                clearBtn.removeAttribute('onclick');
                clearBtn.addEventListener('click', clearCurrentTask);
            }
            
            console.log('模态框按钮事件已绑定');
        }
    }, 100);
}

/**
 * 更新专注状态显示
 */
function updateFocusStatus(payload) {
    // 处理事件数据格式
    let newState;
    if (typeof payload === 'string') {
        newState = payload;
    } else if (payload && payload.state) {
        newState = payload.state;
        console.log(`专注状态更新: ${newState} (置信度: ${payload.confidence || 'N/A'})`);
        if (payload.application_name) {
            console.log(`当前应用: ${payload.application_name}`);
        }
    } else {
        console.warn('无效的专注状态数据:', payload);
        return;
    }

    currentFocusState = newState;
    const config = statusConfig[newState];
    
    if (!config) {
        console.warn('未知的专注状态:', newState);
        return;
    }
    
    const statusCircle = document.getElementById('status-circle');
    const statusIcon = document.getElementById('status-icon');
    const statusText = document.getElementById('status-text');
    
    if (statusCircle && statusIcon && statusText) {
        // 清除所有可能的类
        statusCircle.className = 'w-40 h-40 rounded-full flex items-center justify-center mb-4';
        statusIcon.className = config.icon + ' text-6xl';
        
        // 应用新的样式
        statusCircle.classList.add(config.bgColor, config.borderColor, 'border-4');
        statusIcon.classList.add(config.textColor);
        statusText.textContent = config.text;
        statusText.className = 'text-3xl font-bold ' + config.statusTextColor;
        
        // 添加动画效果
        if (config.animation) {
            statusCircle.classList.add(config.animation);
        }
        
        console.log(`专注状态界面已更新为: ${config.text}`);
    } else {
        console.warn('找不到专注状态显示元素');
    }
}

/**
 * 打开任务选择器
 */
function openTaskSelector() {
    console.log('打开任务选择器'); // 调试日志
    
    const modal = document.getElementById('task-selection-modal');
    const modalTaskList = document.getElementById('modal-task-list');
    
    if (!modal || !modalTaskList) {
        console.error('找不到模态框元素');
        return;
    }
    
    // 清空模态框任务列表
    modalTaskList.innerHTML = '';
    
    // 获取当前任务列表
    const taskList = document.getElementById('task-list');
    if (!taskList) {
        console.error('找不到任务列表元素');
        return;
    }
    
    const taskItems = taskList.querySelectorAll('[data-task-id]');
    console.log('找到任务数量:', taskItems.length);
    
    if (taskItems.length > 0) {
        let hasUncompletedTasks = false;
        
        taskItems.forEach(taskItem => {
            const taskId = taskItem.getAttribute('data-task-id');
            const taskLabel = taskItem.querySelector('label');
            const taskCheckbox = taskItem.querySelector('input[type="checkbox"]');
            
            if (taskLabel && taskCheckbox && !taskCheckbox.checked) { // 只显示未完成的任务
                hasUncompletedTasks = true;
                
                const modalTaskItem = document.createElement('div');
                modalTaskItem.className = 'p-3 bg-gray-700 hover:bg-gray-600 rounded-lg cursor-pointer transition-colors flex items-center justify-between';
                modalTaskItem.innerHTML = `
                    <span class="text-gray-200">${taskLabel.textContent}</span>
                    ${currentSelectedTask === taskId ? '<i class="fas fa-check text-green-400"></i>' : ''}
                `;
                
                // 使用事件监听器而不是 onclick 属性
                modalTaskItem.addEventListener('click', () => {
                    selectCurrentTask(taskId, taskLabel.textContent);
                });
                
                modalTaskList.appendChild(modalTaskItem);
            }
        });
        
        // 如果没有未完成的任务
        if (!hasUncompletedTasks) {
            modalTaskList.innerHTML = `
                <div class="text-center text-gray-500 py-8">
                    <i class="fas fa-check-circle text-3xl mb-2 opacity-50"></i>
                    <p>所有任务已完成</p>
                    <p class="text-sm mt-1">添加新任务或查看已完成任务</p>
                </div>
            `;
        }
    } else {
        // 没有任务时的显示
        modalTaskList.innerHTML = `
            <div class="text-center text-gray-500 py-8">
                <i class="fas fa-clipboard-list text-3xl mb-2 opacity-50"></i>
                <p>暂无任务</p>
                <p class="text-sm mt-1">请先在今日任务中添加任务</p>
            </div>
        `;
    }
    
    // 显示模态框
    modal.classList.remove('hidden');
    console.log('模态框已显示');
    
    // 创建并添加键盘事件监听（ESC键关闭）
    modalKeyPressHandler = (e) => {
        if (e.key === 'Escape') {
            closeTaskSelector();
        }
    };
    document.addEventListener('keydown', modalKeyPressHandler);
    
    // 创建并添加点击模态框外部关闭事件
    modalOutsideClickHandler = (e) => {
        if (e.target === modal) {
            closeTaskSelector();
        }
    };
    modal.addEventListener('click', modalOutsideClickHandler);
}

/**
 * 关闭任务选择器
 */
function closeTaskSelector() {
    console.log('关闭任务选择器'); // 调试日志
    
    const modal = document.getElementById('task-selection-modal');
    if (modal) {
        modal.classList.add('hidden');
        
        // 清理事件监听器
        if (modalOutsideClickHandler) {
            modal.removeEventListener('click', modalOutsideClickHandler);
            modalOutsideClickHandler = null;
        }
        
        if (modalKeyPressHandler) {
            document.removeEventListener('keydown', modalKeyPressHandler);
            modalKeyPressHandler = null;
        }
        
        console.log('模态框已隐藏，事件监听器已清理');
    }
}

/**
 * 选择当前任务
 */
function selectCurrentTask(taskId, taskText) {
    console.log('选择任务:', taskId, taskText); // 调试日志
    
    if (!taskId || !taskText) {
        console.error('无效的任务参数');
        return;
    }
    
    // 防抖处理，避免重复点击
    if (selectCurrentTask.isProcessing) {
        console.log('正在处理中，跳过重复点击');
        return;
    }
    selectCurrentTask.isProcessing = true;
    
    try {
        currentSelectedTask = taskId;
        updateCurrentTaskDisplay(taskText);
        
        // 保存到本地存储
        localStorage.setItem('currentSelectedTask', JSON.stringify({
            id: taskId,
            text: taskText
        }));
        
        // 同步到计时器界面
        updateTimerCurrentTask(taskText);
        
        closeTaskSelector();
        showNotification('任务已选择', `当前任务设置为: ${taskText}`);
        
        console.log('任务选择完成');
    } catch (error) {
        console.error('选择任务失败:', error);
        showNotification('选择失败', '任务选择失败，请重试');
    } finally {
        // 延迟重置防抖标志
        setTimeout(() => {
            selectCurrentTask.isProcessing = false;
        }, 500);
    }
}

/**
 * 清除当前任务
 */
function clearCurrentTask() {
    console.log('清除当前任务'); // 调试日志
    
    try {
        currentSelectedTask = null;
        updateCurrentTaskDisplay();
        
        // 清除本地存储
        localStorage.removeItem('currentSelectedTask');
        
        // 同步到计时器界面
        updateTimerCurrentTask();
        
        closeTaskSelector();
        showNotification('任务已清除', '当前任务已清除');
        
        console.log('任务清除完成');
    } catch (error) {
        console.error('清除任务失败:', error);
        showNotification('清除失败', '任务清除失败，请重试');
    }
}

/**
 * 更新当前任务显示
 */
function updateCurrentTaskDisplay(taskText = null) {
    const currentTaskTextElement = document.getElementById('current-task-text');
    
    if (!currentTaskTextElement) {
        console.warn('找不到当前任务显示元素');
        return;
    }
    
    if (taskText) {
        // 验证任务是否仍然存在
        if (currentSelectedTask) {
            const taskList = document.getElementById('task-list');
            if (taskList) {
                const taskElement = taskList.querySelector(`[data-task-id="${currentSelectedTask}"]`);
                const taskCheckbox = taskElement?.querySelector('input[type="checkbox"]');
                
                // 如果任务不存在或已完成，清除选择
                if (!taskElement || (taskCheckbox && taskCheckbox.checked)) {
                    console.log('任务已不存在或已完成，清除选择');
                    currentSelectedTask = null;
                    localStorage.removeItem('currentSelectedTask');
                    taskText = null;
                }
            }
        }
    }
    
    if (taskText) {
        currentTaskTextElement.textContent = taskText;
        currentTaskTextElement.className = 'text-blue-300 font-medium';
    } else {
        currentTaskTextElement.textContent = '暂无任务';
        currentTaskTextElement.className = 'text-gray-300 font-medium';
    }
}

/**
 * 同步当前任务到计时器界面
 */
function updateTimerCurrentTask(taskText = null) {
    const timerCurrentTaskDisplay = document.getElementById('current-task-display');
    
    if (timerCurrentTaskDisplay) {
        if (taskText) {
            timerCurrentTaskDisplay.textContent = `当前任务: ${taskText}`;
            timerCurrentTaskDisplay.className = 'text-blue-400 mt-2';
        } else {
            timerCurrentTaskDisplay.textContent = '当前任务: 暂无任务';
            timerCurrentTaskDisplay.className = 'text-gray-400 mt-2';
        }
    }
}

/**
 * 加载保存的当前任务
 */
function loadCurrentTask() {
    const savedTask = localStorage.getItem('currentSelectedTask');
    if (savedTask) {
        try {
            const task = JSON.parse(savedTask);
            currentSelectedTask = task.id;
            updateCurrentTaskDisplay(task.text);
            updateTimerCurrentTask(task.text);
        } catch (error) {
            console.error('加载当前任务失败:', error);
        }
    }
}

/**
 * 切换监控状态
 */
async function toggleMonitoring() {
    try {
        if (isMonitoring) {
            // 停止监控
            await TauriAPI.stopMonitoring();
            isMonitoring = false;
            updateFocusStatus('idle');
            showNotification('监控已停止', '应用监控已关闭', 'info', true);
        } else {
            // 开始监控前进行全面检查
            console.log('开始监控前检查...');
            
            // 1. 检查AI配置
            const aiConfigValid = await checkAIConfiguration();
            if (!aiConfigValid) {
                return; // 配置无效，已显示错误提示，直接返回
            }
            
            // 2. 检查白名单黑名单配置
            const listConfigValid = await checkWhiteBlacklistConfiguration();
            if (!listConfigValid) {
                return; // 配置不完整，显示提示
            }
            
            // 3. 测试API连接
            const apiConnected = await testAPIConnectionForMonitoring();
            if (!apiConnected) {
                return; // API连接失败，已显示错误提示
            }
            
            // 所有检查通过，保存监控配置并开始监控
            await saveMonitoringConfigForStart();
            await TauriAPI.startMonitoring();
            isMonitoring = true;
            updateFocusStatus('focused');
            showNotification('监控已开始', '🎯 所有系统检查通过，监控已启动！', 'success', true);
        }
        updateMonitoringButton();
    } catch (error) {
        console.error('监控状态切换失败:', error);
        showNotification('操作失败', `监控操作失败: ${error.message || error}`);
        
        // 确保状态一致性
        isMonitoring = false;
        updateFocusStatus('idle');
        updateMonitoringButton();
    }
}

/**
 * 检查AI配置是否有效
 */
async function checkAIConfiguration() {
    try {
        const aiConfig = await TauriAPI.loadAIConfig();
        
        if (!aiConfig.api_key || aiConfig.api_key.trim() === '') {
            showNotification('配置错误', '⚠️ AI API密钥未配置，请前往设置页面配置API密钥', 'error');
            // 自动切换到设置页面
            setTimeout(() => {
                document.querySelector('[data-target="settings"]').click();
            }, 2000);
            return false;
        }
        
        if (!aiConfig.detection_model || aiConfig.detection_model.trim() === '') {
            showNotification('配置错误', '⚠️ 检测模型未选择，请前往设置页面选择AI模型', 'error');
            setTimeout(() => {
                document.querySelector('[data-target="settings"]').click();
            }, 2000);
            return false;
        }
        
        console.log('AI配置检查通过');
        return true;
        
    } catch (error) {
        console.error('AI配置检查失败:', error);
        showNotification('配置检查失败', '无法读取AI配置，请检查设置', 'error');
        return false;
    }
}

/**
 * 检查白名单黑名单配置
 */
async function checkWhiteBlacklistConfiguration() {
    try {
        const settings = await TauriAPI.loadUserSettings();
        
        const hasWhitelist = settings.whitelist && settings.whitelist.length > 0;
        const hasBlacklist = settings.blacklist && settings.blacklist.length > 0;
        
        if (!hasWhitelist && !hasBlacklist) {
            showNotification('配置提醒', '🔧 建议配置白名单或黑名单以获得更准确的监控结果\n\n点击"前往设置"或继续启动监控', 'warning');
            
            // 显示确认对话框
            const confirmed = confirm('未配置白名单或黑名单，监控准确度可能降低。\n\n是否继续启动监控？\n\n点击"取消"前往设置页面配置。');
            
            if (!confirmed) {
                document.querySelector('[data-target="settings"]').click();
                return false;
            }
        }
        
        console.log('白名单黑名单配置检查完成');
        return true;
        
    } catch (error) {
        console.error('白名单黑名单配置检查失败:', error);
        return true; // 不阻止监控启动
    }
}

/**
 * 测试API连接（用于监控启动检查）
 */
async function testAPIConnectionForMonitoring() {
    try {
        showNotification('正在检查', '🔍 正在测试AI API连接...', 'info');
        
        const aiConfig = await TauriAPI.loadAIConfig();
        const testResult = await TauriAPI.testAIAPI(aiConfig);
        
        if (!testResult.success) {
            showNotification('API连接失败', `❌ ${testResult.message}\n\n请检查API密钥和网络连接`, 'error');
            
            // 提供重试选项
            const retry = confirm(`API连接失败: ${testResult.message}\n\n是否前往设置页面检查配置？`);
            if (retry) {
                document.querySelector('[data-target="settings"]').click();
            }
            return false;
        }
        
        console.log('API连接测试通过:', testResult);
        return true;
        
    } catch (error) {
        console.error('API连接测试失败:', error);
        showNotification('连接测试失败', `❌ API连接测试异常: ${error.message || error}`, 'error');
        return false;
    }
}

/**
 * 更新监控按钮显示
 */
function updateMonitoringButton() {
    const button = document.getElementById('monitoring-toggle-btn');
    const icon = button.querySelector('i');
    const text = button.querySelector('span');
    
    if (isMonitoring) {
        button.className = 'bg-red-600 hover:bg-red-700 text-white font-bold py-3 px-8 rounded-lg transition-transform duration-200 hover:scale-105 flex items-center space-x-2';
        icon.className = 'fas fa-stop-circle';
        text.textContent = '停止监控';
    } else {
        button.className = 'bg-green-600 hover:bg-green-700 text-white font-bold py-3 px-8 rounded-lg transition-transform duration-200 hover:scale-105 flex items-center space-x-2';
        icon.className = 'fas fa-play-circle';
        text.textContent = '开始监控';
    }
}

/**
 * 暂停15分钟功能
 */
function pauseFifteenMinutes() {
    if (pauseTimer) {
        // 如果已经在暂停中，取消暂停
        clearInterval(pauseTimer);
        pauseTimer = null;
        pauseCountdown = 0;
        updatePauseButton();
        showNotification('暂停已取消', '专注监控已恢复');
        return;
    }

    // 开始15分钟倒计时
    pauseCountdown = 15 * 60; // 15分钟 = 900秒
    
    pauseTimer = setInterval(() => {
        pauseCountdown--;
        updatePauseButton();
        
        if (pauseCountdown <= 0) {
            clearInterval(pauseTimer);
            pauseTimer = null;
            updatePauseButton();
            showNotification('暂停结束', '15分钟休息时间结束，继续专注！');
        }
    }, 1000);
    
    showNotification('暂停开始', '开始15分钟休息时间');
}

/**
 * 更新暂停按钮显示
 */
function updatePauseButton() {
    const button = document.getElementById('pause-15-btn');
    const text = document.getElementById('pause-text');
    
    if (pauseTimer && pauseCountdown > 0) {
        const minutes = Math.floor(pauseCountdown / 60);
        const seconds = pauseCountdown % 60;
        text.textContent = `剩余 ${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
        button.className = 'bg-yellow-600 hover:bg-yellow-700 text-white font-bold py-3 px-8 rounded-lg transition-transform duration-200 hover:scale-105';
    } else {
        text.textContent = '暂停15分钟';
        button.className = 'bg-gray-600 hover:bg-gray-500 text-white font-bold py-3 px-8 rounded-lg transition-transform duration-200 hover:scale-105';
    }
}

/**
 * 删除任务
 */
async function deleteTask(taskId) {
    if (!taskId) {
        console.error('删除任务失败: 任务ID为空');
        return;
    }

    try {
        // 从后端删除任务
        await TauriAPI.deleteTask(taskId);
        console.log(`任务 ${taskId} 已删除`);

        // 从DOM中移除任务元素
        const taskElement = document.querySelector(`[data-task-id="${taskId}"]`);
        if (taskElement) {
            taskElement.remove();
        }

        // 检查是否需要显示空任务提示
        checkAndShowEmptyTaskMessage();

        // 如果删除的是当前选中的任务，清除当前任务
        if (currentSelectedTask === taskId) {
            clearCurrentTask();
        }

        // 同步任务选择器
        updateTaskSelector();

        showNotification('任务已删除', '任务删除成功');

    } catch (error) {
        console.error('删除任务失败:', error);
        showNotification('删除失败', '任务删除失败，请重试');
    }
}

/**
 * 隐藏空任务提示
 */
function hideEmptyTaskMessage() {
    const emptyMessage = document.getElementById('empty-task-message');
    if (emptyMessage) {
        emptyMessage.style.display = 'none';
    }
}

/**
 * 显示空任务提示
 */
function showEmptyTaskMessage() {
    const emptyMessage = document.getElementById('empty-task-message');
    if (emptyMessage) {
        emptyMessage.style.display = 'block';
    }
}

/**
 * 检查并显示空任务提示
 */
function checkAndShowEmptyTaskMessage() {
    const taskList = document.getElementById('task-list');
    const tasks = taskList.querySelectorAll('[data-task-id]');
    
    if (tasks.length === 0) {
        showEmptyTaskMessage();
    }
}

/**
 * 更新任务选择器
 */
function updateTaskSelector() {
    const taskSelector = document.getElementById('task-selector');
    const currentTaskDisplay = document.getElementById('current-task-display');
    
    if (!taskSelector) return;
    
    // 清空现有选项
    taskSelector.innerHTML = '<option value="">暂无任务</option>';
    
    // 获取任务列表
    const taskList = document.getElementById('task-list');
    if (taskList) {
        const taskItems = taskList.querySelectorAll('[data-task-id]');
        
        if (taskItems.length > 0) {
            taskItems.forEach(taskItem => {
                const taskId = taskItem.getAttribute('data-task-id');
                const taskLabel = taskItem.querySelector('label');
                const taskCheckbox = taskItem.querySelector('input[type="checkbox"]');
                
                if (taskLabel && !taskCheckbox.checked) { // 只显示未完成的任务
                    const option = document.createElement('option');
                    option.value = taskId;
                    option.textContent = taskLabel.textContent;
                    taskSelector.appendChild(option);
                }
            });
        }
    }
    
    // 监听任务选择变化
    taskSelector.addEventListener('change', function() {
        updateSelectedTask(this.value);
    });
    
    // 如果没有任务，更新当前任务显示
    if (taskSelector.children.length === 1) {
        updateSelectedTask('');
    }
}

/**
 * 更新选中的任务
 */
function updateSelectedTask(taskId) {
    const currentTaskDisplay = document.getElementById('current-task-display');
    const taskSelector = document.getElementById('task-selector');
    
    if (!currentTaskDisplay) return;
    
    if (taskId && taskId !== '') {
        const selectedOption = taskSelector.querySelector(`option[value="${taskId}"]`);
        if (selectedOption) {
            currentTaskDisplay.textContent = `当前任务: ${selectedOption.textContent}`;
            currentTaskDisplay.className = 'text-blue-400 mt-2';
        }
    } else {
        currentTaskDisplay.textContent = '当前任务: 暂无任务';
        currentTaskDisplay.className = 'text-gray-400 mt-2';
    }
}

/**
 * 验证当前任务是否存在
 */
function validateCurrentTask() {
    const taskSelector = document.getElementById('task-selector');
    const currentTaskDisplay = document.getElementById('current-task-display');

    if (!taskSelector || !currentTaskDisplay) return;

    const selectedTaskId = taskSelector.value;
    const taskList = document.getElementById('task-list');

    if (selectedTaskId && selectedTaskId !== '') {
        const taskItem = taskList.querySelector(`[data-task-id="${selectedTaskId}"]`);
        if (!taskItem) {
            // 如果选中的任务不存在，则清除当前任务
            taskSelector.value = '';
            updateSelectedTask('');
            updateCurrentTaskDisplay();
            updateTimerCurrentTask();
            showNotification('任务已清除', '当前任务已清除');
        }
    } else {
        // 如果没有任务选中，则清除当前任务
        updateCurrentTaskDisplay();
        updateTimerCurrentTask();
    }
}

/**
 * 防抖函数
 */
function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

/**
 * 初始化主题切换功能
 */
function initThemeToggle() {
    const themeToggle = document.getElementById('theme-toggle');
    
    if (themeToggle) {
        // 绑定切换事件
        themeToggle.addEventListener('change', toggleTheme);
        console.log('主题切换功能已初始化');
    } else {
        console.warn('找不到主题切换开关元素');
    }
    
    // 页面加载时应用保存的主题
    loadSavedTheme();
}

/**
 * 切换主题
 */
function toggleTheme() {
    const themeToggle = document.getElementById('theme-toggle');
    const body = document.body;
    
    if (!themeToggle || !body) return;
    
    try {
        const isDark = themeToggle.checked;
        const newTheme = isDark ? 'dark' : 'light';
        
        // 应用新主题
        applyTheme(newTheme);
        
        // 保存主题选择
        saveTheme(newTheme);
        
        // 显示通知
        const themeText = isDark ? '深色模式' : '浅色模式';
        showNotification('主题已切换', `已切换到${themeText}`);
        
        console.log(`主题已切换为: ${newTheme}`);
        
    } catch (error) {
        console.error('主题切换失败:', error);
        showNotification('切换失败', '主题切换失败，请重试');
    }
}

/**
 * 应用主题
 */
function applyTheme(theme) {
    const body = document.body;
    
    if (!body) return;
    
    // 设置data-theme属性
    body.setAttribute('data-theme', theme);
    
    // 更新切换开关状态（不触发事件）
    const themeToggle = document.getElementById('theme-toggle');
    if (themeToggle) {
        themeToggle.removeEventListener('change', toggleTheme);
        themeToggle.checked = theme === 'dark';
        themeToggle.addEventListener('change', toggleTheme);
    }
    
    // 触发自定义事件，供其他组件监听
    const themeChangeEvent = new CustomEvent('themeChanged', {
        detail: { theme }
    });
    document.dispatchEvent(themeChangeEvent);
    
    console.log(`主题已应用: ${theme}`);
}

/**
 * 保存主题到本地存储
 */
function saveTheme(theme) {
    try {
        localStorage.setItem('myFocusTheme', theme);
        
        // 同时保存到用户设置
        const settings = {
            theme: theme,
            // 获取其他现有设置
            whitelist: getWhitelistItems(),
            blacklist: getBlacklistItems(),
            autostart: document.getElementById('autostart')?.checked || false,
            fatigue_notify: document.getElementById('fatigue-notify')?.checked || false,
            
            // 分心干预设置
            distraction_intervention: {
                enabled: document.getElementById('distraction-intervention-enabled')?.checked || true,
                light_distraction_notification: document.getElementById('light-distraction-notification')?.checked || true,
                severe_distraction_popup: document.getElementById('severe-distraction-popup')?.checked || true,
                encouragement_enabled: document.getElementById('encouragement-enabled')?.checked || true,
                intervention_cooldown_minutes: parseInt(document.getElementById('intervention-cooldown')?.value || '5'),
                notification_sound: document.getElementById('intervention-sound')?.checked || true,
                popup_duration_seconds: parseInt(document.getElementById('popup-duration')?.value || '10'),
                encouragement_frequency: document.getElementById('encouragement-frequency')?.value || 'medium'
            }
        };
        
        // 静默保存用户设置（包含主题）
        saveUserSettingsToBackendSilently(settings);
        
        console.log(`主题已保存: ${theme}`);
        
    } catch (error) {
        console.error('保存主题失败:', error);
    }
}

/**
 * 静默保存用户设置到后端（不显示通知）
 */
async function saveUserSettingsToBackendSilently(settings) {
    try {
        await TauriAPI.saveUserSettings(settings);
        console.log('用户设置（包含主题）已静默保存到后端');
        
        // 同时保存到本地存储作为备份
        localStorage.setItem('myFocusSettings', JSON.stringify(settings));
    } catch (error) {
        console.error('静默保存用户设置到后端失败:', error);
        // 如果后端保存失败，至少保存到本地存储
        localStorage.setItem('myFocusSettings', JSON.stringify(settings));
    }
}

/**
 * 加载保存的主题
 */
function loadSavedTheme() {
    try {
        // 首先尝试从本地存储读取
        let savedTheme = localStorage.getItem('myFocusTheme');
        
        // 如果本地存储没有，尝试从用户设置读取
        if (!savedTheme) {
            const savedSettings = localStorage.getItem('myFocusSettings');
            if (savedSettings) {
                const settings = JSON.parse(savedSettings);
                savedTheme = settings.theme;
            }
        }
        
        // 如果还是没有，使用默认深色主题
        if (!savedTheme) {
            savedTheme = 'dark';
        }
        
        console.log(`加载保存的主题: ${savedTheme}`);
        
        // 应用主题
        applyTheme(savedTheme);
        
        // 异步从后端加载完整的用户设置并同步主题
        loadUserSettingsAndSyncTheme();
        
    } catch (error) {
        console.error('加载保存的主题失败:', error);
        // 出错时使用默认深色主题
        applyTheme('dark');
    }
}

/**
 * 从后端加载用户设置并同步主题
 */
async function loadUserSettingsAndSyncTheme() {
    try {
        const settings = await TauriAPI.loadUserSettings();
        
        if (settings && settings.theme) {
            console.log(`从后端同步主题: ${settings.theme}`);
            applyTheme(settings.theme);
        }
        
    } catch (error) {
        console.error('从后端同步主题失败:', error);
        // 不影响应用继续运行
    }
}

/**
 * 获取当前主题
 */
function getCurrentTheme() {
    const body = document.body;
    return body ? body.getAttribute('data-theme') || 'dark' : 'dark';
}

/**
 * 监听主题变化事件（供其他组件使用）
 */
function onThemeChanged(callback) {
    document.addEventListener('themeChanged', callback);
}

/**
 * 移除主题变化监听器
 */
function offThemeChanged(callback) {
    document.removeEventListener('themeChanged', callback);
}

/**
 * 批量DOM更新优化
 */
function batchDOMUpdates(updates) {
    requestAnimationFrame(() => {
        updates.forEach(update => {
            try {
                update();
            } catch (error) {
                console.error('DOM更新失败:', error);
            }
        });
    });
}

/**
 * 安全的元素查找
 */
function safeQuerySelector(selector, parent = document) {
    try {
        return parent.querySelector(selector);
    } catch (error) {
        console.error('元素查找失败:', selector, error);
        return null;
    }
}

// 导出函数供外部使用
window.MyFocus = {
    showNotification,
    formatTime,
    getTodayString
};

// 导出控制函数到全局作用域
window.toggleMonitoring = toggleMonitoring;
window.pauseFifteenMinutes = pauseFifteenMinutes;
window.deleteTask = deleteTask;
window.openTaskSelector = openTaskSelector;
window.closeTaskSelector = closeTaskSelector;
window.clearCurrentTask = clearCurrentTask;
window.selectCurrentTask = selectCurrentTask;
window.removeWhitelistItem = removeWhitelistItem;
window.removeBlacklistItem = removeBlacklistItem;

/**
 * 保存监控配置用于启动监控
 */
async function saveMonitoringConfigForStart() {
    try {
        console.log('正在准备监控配置...');
        
        // 获取AI配置
        const aiConfig = await TauriAPI.loadAIConfig();
        console.log('加载AI配置:', aiConfig);
        
        // 获取用户设置（白名单黑名单）
        const userSettings = await TauriAPI.loadUserSettings();
        console.log('加载用户设置:', userSettings);
        
        // 获取监控间隔设置
        const intervalSelect = document.getElementById('monitoring-interval');
        const intervalMinutes = intervalSelect ? parseInt(intervalSelect.value) : 3;
        
        // 创建监控配置
        const monitoringConfig = {
            enabled: true, // 重要：启用监控
            interval_minutes: intervalMinutes,
            whitelist: userSettings.whitelist || [],
            blacklist: userSettings.blacklist || [],
            ai_config: aiConfig
        };
        
        console.log('准备保存的监控配置:', monitoringConfig);
        
        // 保存监控配置到后端
        await TauriAPI.saveMonitoringConfig(monitoringConfig);
        console.log('监控配置已保存');
        
    } catch (error) {
        console.error('保存监控配置失败:', error);
        throw new Error(`监控配置保存失败: ${error.message || error}`);
    }
}

// 监控配置相关函数
window.applyMonitoringInterval = applyMonitoringInterval;
window.triggerManualCheck = triggerManualCheck;

/**
 * 应用监控频率设置
 */
async function applyMonitoringInterval() {
    const intervalSelect = document.getElementById('monitoring-interval');
    const applyBtn = document.getElementById('apply-interval-btn');
    
    if (!intervalSelect || !applyBtn) return;
    
    const intervalMinutes = parseInt(intervalSelect.value);
    
    // 显示加载状态
    applyBtn.disabled = true;
    applyBtn.innerHTML = '<i class="fas fa-spinner fa-spin"></i>';
    
    try {
        await TauriAPI.updateMonitoringInterval(intervalMinutes);
        
        showNotification('设置已更新', `监控频率已设置为每${intervalMinutes}分钟`);
        console.log('监控频率已更新:', intervalMinutes);
        
    } catch (error) {
        console.error('更新监控频率失败:', error);
        showNotification('设置失败', '监控频率更新失败，请稍后重试');
    } finally {
        // 恢复按钮状态
        applyBtn.disabled = false;
        applyBtn.innerHTML = '应用';
    }
}

/**
 * 触发手动检查
 */
async function triggerManualCheck() {
    const checkBtn = document.getElementById('manual-check-btn');
    
    if (!checkBtn) return;
    
    // 显示加载状态
    checkBtn.disabled = true;
    checkBtn.innerHTML = '<i class="fas fa-spinner fa-spin mr-2"></i><span>检查中...</span>';
    
    try {
        const result = await TauriAPI.triggerMonitoringCheck();
        console.log('手动检查结果:', result);
        
        // 更新仪表盘状态显示
        updateDashboardFocusState(result);
        
        // 显示检查结果
        showMonitoringResult(result);
        
        showNotification('检查完成', '专注状态检查已完成');
        
    } catch (error) {
        console.error('手动检查失败:', error);
        showNotification('检查失败', '无法执行专注状态检查，请稍后重试');
    } finally {
        // 恢复按钮状态
        checkBtn.disabled = false;
        checkBtn.innerHTML = '<i class="fas fa-search mr-2"></i><span>立即检查专注状态</span>';
    }
}

/**
 * 更新仪表盘专注状态显示
 */
function updateDashboardFocusState(result) {
    const focusStateMap = {
        'Focused': 'focused',
        'Distracted': 'distracted', 
        'SeverelyDistracted': 'severely_distracted',
        'Unknown': 'idle'
    };
    
    const mappedState = focusStateMap[result.focus_state] || 'idle';
    updateFocusStatus(mappedState);
    
    // 更新当前应用显示
    if (result.application_name) {
        console.log(`当前应用: ${result.application_name}`);
        if (result.window_title) {
            console.log(`窗口标题: ${result.window_title}`);
        }
    }
}

/**
 * 显示监控结果详情
 */
function showMonitoringResult(result) {
    // 创建结果显示模态框
    const modal = document.createElement('div');
    modal.className = 'fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50';
    modal.innerHTML = `
        <div class="bg-gray-800 rounded-xl p-6 w-96 max-w-md mx-4">
            <h3 class="text-lg font-semibold text-white mb-4 flex items-center">
                <i class="fas fa-brain mr-2 text-blue-400"></i>
                专注状态检查结果
            </h3>
            <div class="space-y-3">
                <div class="flex items-center justify-between">
                    <span class="text-gray-300">状态:</span>
                    <span class="font-bold ${getStateColor(result.focus_state)}">${getStateText(result.focus_state)}</span>
                </div>
                <div class="flex items-center justify-between">
                    <span class="text-gray-300">置信度:</span>
                    <span class="text-white">${Math.round(result.confidence * 100)}%</span>
                </div>
                ${result.application_name ? `
                <div class="flex items-center justify-between">
                    <span class="text-gray-300">当前应用:</span>
                    <span class="text-white text-sm">${result.application_name}</span>
                </div>
                ` : ''}
                ${result.ai_analysis ? `
                <div class="mt-4">
                    <span class="text-gray-300 text-sm">AI分析:</span>
                    <p class="text-gray-200 text-sm mt-1 bg-gray-700 p-2 rounded">${result.ai_analysis}</p>
                </div>
                ` : ''}
            </div>
            <div class="mt-6 flex justify-end">
                <button onclick="this.closest('.fixed').remove()" class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-lg">
                    确定
                </button>
            </div>
        </div>
    `;
    
    document.body.appendChild(modal);
    
    // 3秒后自动关闭
    setTimeout(() => {
        if (modal.parentNode) {
            modal.remove();
        }
    }, 5000);
}

/**
 * 获取状态对应的颜色类
 */
function getStateColor(state) {
    const colorMap = {
        'Focused': 'text-green-400',
        'Distracted': 'text-yellow-400',
        'SeverelyDistracted': 'text-red-400',
        'Unknown': 'text-gray-400'
    };
    return colorMap[state] || 'text-gray-400';
}

/**
 * 获取状态对应的中文文本
 */
function getStateText(state) {
    const textMap = {
        'Focused': '专注',
        'Distracted': '分心', 
        'SeverelyDistracted': '严重分心',
        'Unknown': '未知'
    };
    return textMap[state] || '未知';
}

/**
 * 显示分心干预弹窗
 */
function showDistractionIntervention(payload) {
    console.log('显示分心干预弹窗:', payload);
    
    const { type, message, urgent, duration_seconds, sound_enabled } = payload;
    
    // 发送系统通知（仅分心干预，鼓励消息只显示应用内通知）
    let title, notificationType, showSystemNotif;
    
    if (type === 'encouragement') {
        // 专注鼓励消息 - 仅应用内通知
        title = '🎉 专注表现优秀';
        notificationType = 'success';
        showSystemNotif = false;
    } else if (urgent) {
        // 严重分心警告 - 显示系统通知
        title = '🚨 严重分心警告';
        notificationType = 'error';
        showSystemNotif = true;
    } else {
        // 轻度分心提醒 - 显示系统通知
        title = '⚠️ 专注提醒';
        notificationType = 'warning';
        showSystemNotif = true;
    }
    
    showNotification(title, message, notificationType, showSystemNotif);
    
    // 创建弹窗HTML
    const modalHtml = `
        <div id="distraction-modal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" style="z-index: 10000;">
            <div class="bg-white rounded-lg p-8 max-w-md mx-4 text-center relative transform animate-bounce">
                <div class="mb-6">
                    <div class="text-6xl mb-4">
                        ${type === 'encouragement' ? '🎉' : (urgent ? '🚨' : '⚠️')}
                    </div>
                    <h2 class="text-2xl font-bold ${type === 'encouragement' ? 'text-green-600' : (urgent ? 'text-red-600' : 'text-orange-500')} mb-2">
                        ${type === 'encouragement' ? '专注表现优秀' : (urgent ? '严重分心警告' : '专注提醒')}
                    </h2>
                    <p class="text-gray-700 text-lg leading-relaxed">
                        ${message}
                    </p>
                </div>
                
                <div class="flex gap-4 justify-center">
                    <button onclick="dismissDistractionModal()" 
                            class="px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors font-medium">
                        知道了
                    </button>
                    ${urgent ? `
                    <button onclick="dismissDistractionModal(); pauseMonitoring();" 
                            class="px-6 py-3 bg-gray-500 text-white rounded-lg hover:bg-gray-600 transition-colors font-medium">
                        暂停监控5分钟
                    </button>
                    ` : ''}
                </div>
                
                <div class="mt-4 text-sm text-gray-500">
                    ${duration_seconds}秒后自动关闭
                </div>
            </div>
        </div>
    `;
    
    // 移除现有的干预弹窗
    const existingModal = document.getElementById('distraction-modal');
    if (existingModal) {
        existingModal.remove();
    }
    
    // 添加新弹窗到页面
    document.body.insertAdjacentHTML('beforeend', modalHtml);
    
    // 播放提醒音效（如果启用）
    if (sound_enabled) {
        playNotificationSound(urgent);
    }
    
    // 自动关闭计时器
    setTimeout(() => {
        dismissDistractionModal();
    }, (duration_seconds || 10) * 1000);
    
    // 记录干预显示日志
    console.log(`分心干预弹窗已显示: ${type}, 紧急程度: ${urgent ? '高' : '低'}`);
}

/**
 * 关闭分心干预弹窗
 */
function dismissDistractionModal() {
    const modal = document.getElementById('distraction-modal');
    if (modal) {
        modal.classList.add('animate-fadeOut');
        setTimeout(() => {
            modal.remove();
        }, 300);
    }
}

/**
 * 播放通知音效
 */
function playNotificationSound(urgent = false) {
    try {
        // 创建音频上下文播放简单的提示音
        const audioContext = new (window.AudioContext || window.webkitAudioContext)();
        const oscillator = audioContext.createOscillator();
        const gainNode = audioContext.createGain();
        
        oscillator.connect(gainNode);
        gainNode.connect(audioContext.destination);
        
        // 设置音频参数
        oscillator.frequency.value = urgent ? 800 : 600; // 紧急时频率更高
        oscillator.type = 'sine';
        
        gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
        gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.5);
        
        oscillator.start(audioContext.currentTime);
        oscillator.stop(audioContext.currentTime + 0.5);
        
        console.log(`播放${urgent ? '紧急' : '普通'}提示音`);
    } catch (error) {
        console.warn('播放提示音失败:', error);
    }
}

/**
 * 暂停监控5分钟
 */
async function pauseMonitoring() {
    try {
        await TauriAPI.stopMonitoring();
        console.log('监控已暂停');
        
        // 5分钟后重新启动监控
        setTimeout(async () => {
            try {
                await TauriAPI.startMonitoring();
                console.log('监控已恢复');
            } catch (error) {
                console.error('恢复监控失败:', error);
            }
        }, 5 * 60 * 1000);
        
        // 显示暂停通知
        showTemporaryNotification('监控已暂停5分钟', 'info');
    } catch (error) {
        console.error('暂停监控失败:', error);
    }
}

/**
 * 显示临时通知
 */
function showTemporaryNotification(message, type = 'info') {
    const notification = document.createElement('div');
    notification.className = `fixed top-4 right-4 p-4 rounded-lg text-white z-50 transition-all duration-300 ${
        type === 'info' ? 'bg-blue-500' : 
        type === 'warning' ? 'bg-orange-500' : 
        type === 'error' ? 'bg-red-500' : 'bg-gray-500'
    }`;
    notification.textContent = message;
    
    document.body.appendChild(notification);
    
    setTimeout(() => {
        notification.classList.add('opacity-0', 'translate-x-full');
        setTimeout(() => {
            notification.remove();
        }, 300);
    }, 3000);
} 