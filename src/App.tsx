import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Timer, Play, Square, BarChart3, Settings, Clock, Calendar, Folder, Info, Menu, ChevronLeft, ChevronRight } from 'lucide-react';
import { TimerStatus, Task } from './types';
import Dashboard from './components/Dashboard';
import TaskManagement from './components/TaskManagement';
import CategoryManagement from './components/CategoryManagement';
import Statistics from './components/Statistics';
import SettingsComponent from './components/Settings';
import About from './components/About';
import { ThemeProvider } from './hooks/useTheme';

// 格式化时间函数
const formatDuration = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
};

function App() {
    const [timerStatus, setTimerStatus] = useState<TimerStatus>({
        state: 'stopped',
        elapsed_seconds: 0,
        total_today_seconds: 0,
    });

    const [tasks, setTasks] = useState<Task[]>([]);
    const [activeView, setActiveView] = useState<'dashboard' | 'tasks' | 'categories' | 'statistics' | 'settings' | 'about'>('dashboard');
    const [selectedTaskId, setSelectedTaskId] = useState<string>('');

    // 侧边栏状态管理
    const [sidebarWidth, setSidebarWidth] = useState<number>(256); // 256px 默认宽度
    const [isCollapsed, setIsCollapsed] = useState<boolean>(false);
    const [isDragging, setIsDragging] = useState<boolean>(false);
    const [todayStats, setTodayStats] = useState({
        totalSeconds: 0,
        taskCount: 0,
        averageSeconds: 0,
        efficiency: 85,
        efficiencyDetails: {
            focusScore: 0,
            volumeScore: 0,
            rhythmScore: 0,
            avgSessionMinutes: 0,
            hoursWorked: 0,
            actualSessionsPerHour: 0
        }
    });

    // 获取计时器状态
    const fetchTimerStatus = async () => {
        try {
            const status = await invoke<TimerStatus>('get_timer_status');
            setTimerStatus(status);
        } catch (error) {
            console.error('获取计时器状态失败:', error);
        }
    };

    // 获取任务列表
    const fetchTasks = async () => {
        try {
            const taskList = await invoke<Task[]>('get_tasks');
            setTasks(taskList);
            if (taskList.length > 0 && !selectedTaskId) {
                setSelectedTaskId(taskList[0].id);
            }
        } catch (error) {
            console.error('获取任务列表失败:', error);
        }
    };

    // 获取今日统计数据
    const fetchTodayStats = async () => {
        try {
            // 从后端获取今日真实统计数据
            const todayStats = await invoke<TimerStatus>('get_today_stats');
            console.log('后端今日统计数据:', todayStats);

            const totalSeconds = todayStats.total_today_seconds;
            const taskCount = todayStats.elapsed_seconds; // 复用这个字段传递任务数
            const averageSeconds = taskCount > 0 ? Math.round(totalSeconds / taskCount) : 0;

            // 效率评分计算：多维度综合评估
            let efficiency = 0;
            let focusScore = 0;
            let volumeScore = 0;
            let rhythmScore = 0;
            let avgSessionMinutes = 0;
            let hoursWorked = 0;
            let actualSessionsPerHour = 0;

            if (totalSeconds > 0 && taskCount > 0) {
                hoursWorked = totalSeconds / 3600;
                avgSessionMinutes = (totalSeconds / 60) / taskCount; // 平均每段工作时长（分钟）

                // 1. 专注度评分 (40分) - 基于平均会话时长
                if (avgSessionMinutes >= 25) focusScore = 40;        // 25分钟以上 = 专注
                else if (avgSessionMinutes >= 15) focusScore = 30;   // 15-25分钟 = 良好
                else if (avgSessionMinutes >= 5) focusScore = 20;    // 5-15分钟 = 一般
                else focusScore = 10;                                // 5分钟以下 = 需改进

                // 2. 工作量评分 (30分) - 基于总工作时长
                if (hoursWorked >= 6) volumeScore = 30;              // 6小时以上 = 饱满
                else if (hoursWorked >= 4) volumeScore = 25;         // 4-6小时 = 充实
                else if (hoursWorked >= 2) volumeScore = 20;         // 2-4小时 = 适中
                else if (hoursWorked >= 1) volumeScore = 15;         // 1-2小时 = 轻量
                else volumeScore = 10;                               // 1小时以下 = 起步

                // 3. 节奏评分 (30分) - 基于工作段数与时长的平衡
                const idealSessionsPerHour = 2; // 理想：每小时2段（30分钟一段）

                // 当工作时间少于15分钟时，不计算节奏评分，避免误导性数字
                if (hoursWorked >= 0.25) { // 至少15分钟
                    actualSessionsPerHour = taskCount / hoursWorked;
                    const rhythmRatio = Math.min(actualSessionsPerHour / idealSessionsPerHour, idealSessionsPerHour / actualSessionsPerHour);
                    rhythmScore = Math.round(30 * rhythmRatio);
                } else {
                    // 工作时间太短，按基础分给分
                    actualSessionsPerHour = 0;
                    rhythmScore = 15; // 给予基础分数
                }

                efficiency = Math.min(focusScore + volumeScore + rhythmScore, 100);

                console.log('效率评分详情:', {
                    avgSessionMinutes: avgSessionMinutes.toFixed(1),
                    focusScore,
                    volumeScore,
                    rhythmScore,
                    actualSessionsPerHour: actualSessionsPerHour.toFixed(1),
                    finalEfficiency: efficiency
                });
            }

            console.log('最终统计数据:', {
                totalSeconds,
                taskCount,
                averageSeconds,
                efficiency
            });

            setTodayStats({
                totalSeconds,
                taskCount,
                averageSeconds,
                efficiency,
                efficiencyDetails: {
                    focusScore,
                    volumeScore,
                    rhythmScore,
                    avgSessionMinutes,
                    hoursWorked,
                    actualSessionsPerHour
                }
            });
        } catch (error) {
            console.error('获取统计数据失败:', error);
        }
    };

    // 开始计时
    const startTimer = async (taskId?: string) => {
        const targetTaskId = taskId || selectedTaskId;
        if (!targetTaskId) return;
        try {
            const status = await invoke<TimerStatus>('start_timer', { taskId: targetTaskId });
            setTimerStatus(status);
            if (taskId) setSelectedTaskId(taskId);
        } catch (error) {
            console.error('开始计时失败:', error);
        }
    };

    // 暂停计时
    const pauseTimer = async () => {
        try {
            const status = await invoke<TimerStatus>('pause_timer');
            setTimerStatus(status);
        } catch (error) {
            console.error('暂停计时失败:', error);
        }
    };

    // 停止计时
    const stopTimer = async () => {
        try {
            const status = await invoke<TimerStatus>('stop_timer');
            setTimerStatus(status);
            await fetchTasks(); // 刷新任务列表
            await fetchTimerStatus(); // 重新获取计时器状态以更新今日总时间
            await fetchTodayStats(); // 任务停止后更新统计数据
        } catch (error) {
            console.error('停止计时失败:', error);
        }
    };

    // 侧边栏拖拽处理
    const minSidebarWidth = 60; // 最小宽度（折叠状态）
    const maxSidebarWidth = 350; // 最大宽度（减小从400px到350px）
    const defaultSidebarWidth = 256; // 默认宽度
    const collapseThreshold = 140; // 折叠阈值（增大到140px）
    const minMainContentWidth = 600; // 主体内容最小宽度

    // 计算当前窗口宽度，确保主体内容有足够空间
    const getOptimalSidebarWidth = () => {
        const windowWidth = window.innerWidth;
        const availableWidth = windowWidth - minMainContentWidth;

        if (availableWidth < collapseThreshold) {
            // 空间不足，强制折叠
            return minSidebarWidth;
        } else if (availableWidth < sidebarWidth) {
            // 当前侧栏太宽，压缩到合适大小
            return Math.max(180, Math.min(maxSidebarWidth, availableWidth));
        }

        return sidebarWidth;
    };

    // 窗口大小变化监听
    useEffect(() => {
        const handleResize = () => {
            const optimalWidth = getOptimalSidebarWidth();
            if (optimalWidth !== sidebarWidth) {
                setSidebarWidth(optimalWidth);
                setIsCollapsed(optimalWidth === minSidebarWidth);
            }
        };

        window.addEventListener('resize', handleResize);
        // 初始化时也检查一次
        handleResize();

        return () => {
            window.removeEventListener('resize', handleResize);
        };
    }, [sidebarWidth]);

    const handleMouseDown = (e: React.MouseEvent) => {
        e.preventDefault();
        setIsDragging(true);
        document.body.style.cursor = 'col-resize';
        document.body.style.userSelect = 'none';
    };

    const handleMouseMove = (e: MouseEvent) => {
        if (!isDragging) return;

        // 使用 requestAnimationFrame 优化性能，减少卡顿
        requestAnimationFrame(() => {
            const newWidth = e.clientX;
            const windowWidth = window.innerWidth;
            const maxAllowedWidth = Math.min(maxSidebarWidth, windowWidth - minMainContentWidth);

            if (newWidth < collapseThreshold) {
                // 拖拽到阈值以下时，折叠侧边栏
                if (!isCollapsed) {
                    setIsCollapsed(true);
                    setSidebarWidth(minSidebarWidth);
                }
            } else if (newWidth <= maxAllowedWidth) {
                // 在有效范围内调整宽度
                if (isCollapsed) {
                    setIsCollapsed(false);
                }
                // 确保最小展开宽度为180px，避免过窄的展开状态
                const adjustedWidth = Math.max(180, Math.min(maxAllowedWidth, newWidth));
                setSidebarWidth(adjustedWidth);
            }
        });
    };

    const handleMouseUp = () => {
        setIsDragging(false);
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
    };

    const toggleSidebar = () => {
        if (isCollapsed) {
            setIsCollapsed(false);
            setSidebarWidth(defaultSidebarWidth);
        } else {
            setIsCollapsed(true);
            setSidebarWidth(minSidebarWidth);
        }
    };

    // 优化双击边缘快速切换
    const handleDoubleClick = () => {
        toggleSidebar();
    };

    // 全局鼠标事件监听
    useEffect(() => {
        if (isDragging) {
            document.addEventListener('mousemove', handleMouseMove);
            document.addEventListener('mouseup', handleMouseUp);

            return () => {
                document.removeEventListener('mousemove', handleMouseMove);
                document.removeEventListener('mouseup', handleMouseUp);
            };
        }
    }, [isDragging]);

    // 创建任务功能已移到各个组件中

    // 初始化数据
    useEffect(() => {
        fetchTimerStatus();
        fetchTasks();
    }, []);

    // 只在任务列表变化时更新统计数据（不包括计时器状态变化）
    useEffect(() => {
        fetchTodayStats();
    }, [tasks]);

    // 定时器更新效果
    useEffect(() => {
        let interval: number | null = null;

        if (timerStatus.state === 'running') {
            // 计时器运行时，每秒更新状态
            interval = setInterval(() => {
                fetchTimerStatus();
            }, 1000);
        }

        return () => {
            if (interval) {
                clearInterval(interval);
            }
        };
    }, [timerStatus.state]);

    return (
        <ThemeProvider>
            <div className="h-screen bg-gray-50 dark:bg-gray-900 transition-colors flex flex-col">
                {/* 顶部导航栏 - 固定在顶部 */}
                <nav className="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700 flex-shrink-0 z-10">
                    <div className="px-6 py-4">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center space-x-4">
                                <button
                                    onClick={toggleSidebar}
                                    className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors lg:hidden"
                                    title="切换侧边栏"
                                >
                                    <Menu className="h-5 w-5 text-gray-500 dark:text-gray-400" />
                                </button>
                                <Timer className="h-8 w-8 text-blue-600 dark:text-blue-400" />
                                <h1 className="text-xl font-semibold text-gray-900 dark:text-white">TimeTracker</h1>
                            </div>

                            {/* 计时器状态显示 */}
                            <div className="flex items-center space-x-6">
                                <div className="text-right">
                                    <div className="text-2xl font-mono font-bold text-gray-900 dark:text-white">
                                        {formatDuration(timerStatus.elapsed_seconds)}
                                    </div>
                                    <div className="text-sm text-gray-500 dark:text-gray-400">
                                        今日总计: {formatDuration(timerStatus.total_today_seconds)}
                                    </div>
                                </div>

                                {/* 计时器控制按钮 */}
                                <div className="flex items-center space-x-2">
                                    {timerStatus.state === 'stopped' ? (
                                        <button
                                            onClick={() => startTimer()}
                                            className="flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
                                            disabled={!selectedTaskId}
                                        >
                                            <Play className="h-4 w-4 mr-1" />
                                            开始
                                        </button>
                                    ) : (
                                        <button
                                            onClick={stopTimer}
                                            className="flex items-center px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700"
                                        >
                                            <Square className="h-4 w-4 mr-1" />
                                            停止
                                        </button>
                                    )}
                                </div>
                            </div>
                        </div>
                    </div>
                </nav>

                {/* 主体区域 - 占用剩余空间 */}
                <div className="flex flex-1 overflow-hidden">
                    {/* 侧边栏 - 可调整宽度 */}
                    <div
                        className={`bg-white dark:bg-gray-800 shadow-sm border-r border-gray-200 dark:border-gray-700 flex-shrink-0 relative ${isDragging ? '' : 'transition-all duration-200 ease-out'
                            }`}
                        style={{ width: `${sidebarWidth}px` }}
                    >
                        {/* 侧边栏头部 - 折叠/展开按钮 */}
                        <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                            {!isCollapsed && (
                                <h2 className="text-lg font-semibold text-gray-900 dark:text-white">导航</h2>
                            )}
                            <button
                                onClick={toggleSidebar}
                                className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                                title={isCollapsed ? "展开侧边栏" : "折叠侧边栏"}
                            >
                                {isCollapsed ? (
                                    <ChevronRight className="h-4 w-4 text-gray-500 dark:text-gray-400" />
                                ) : (
                                    <ChevronLeft className="h-4 w-4 text-gray-500 dark:text-gray-400" />
                                )}
                            </button>
                        </div>

                        {/* 导航菜单 */}
                        <nav className="h-full overflow-y-auto">
                            <div className={`p-4 space-y-2 ${isCollapsed ? 'px-2' : ''}`}>
                                {[
                                    { id: 'dashboard', name: '仪表板', icon: BarChart3 },
                                    { id: 'tasks', name: '任务管理', icon: Clock },
                                    { id: 'categories', name: '分类管理', icon: Folder },
                                    { id: 'statistics', name: '统计报告', icon: Calendar },
                                    { id: 'settings', name: '设置', icon: Settings },
                                    { id: 'about', name: '关于', icon: Info },
                                ].map(({ id, name, icon: Icon }) => (
                                    <button
                                        key={id}
                                        onClick={() => setActiveView(id as any)}
                                        className={`w-full flex items-center ${isCollapsed ? 'justify-center px-2' : 'px-4'} py-3 text-sm font-medium rounded-lg transition-all duration-200 ${activeView === id
                                            ? 'bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300'
                                            : 'text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700'
                                            }`}
                                        title={isCollapsed ? name : undefined}
                                    >
                                        <Icon className={`h-5 w-5 ${!isCollapsed ? 'mr-3' : ''}`} />
                                        {!isCollapsed && <span className="truncate">{name}</span>}
                                    </button>
                                ))}
                            </div>
                        </nav>

                        {/* 拖拽调整手柄 */}
                        <div
                            className={`absolute top-0 right-0 w-1 h-full cursor-col-resize transition-all duration-150 ${isDragging
                                ? 'bg-blue-500 shadow-lg'
                                : 'bg-transparent hover:bg-blue-400 hover:shadow-md'
                                }`}
                            onMouseDown={handleMouseDown}
                            onDoubleClick={handleDoubleClick}
                            title="拖拽调整宽度 | 双击切换折叠"
                        >
                            {/* 扩大点击区域 */}
                            <div className="absolute inset-y-0 -right-2 w-5 h-full" />
                            {/* 视觉指示器 */}
                            <div className={`absolute top-1/2 -translate-y-1/2 -right-0.5 w-2 h-8 rounded-full transition-all duration-150 ${isDragging
                                ? 'bg-blue-600 opacity-100'
                                : 'bg-gray-400 dark:bg-gray-500 opacity-0 group-hover:opacity-60'
                                }`} />
                        </div>
                    </div>

                    {/* 主内容区 - 可滚动 */}
                    <div className="flex-1 overflow-y-auto bg-gray-50 dark:bg-gray-900" style={{ minWidth: `${minMainContentWidth}px` }}>
                        <div className="p-8">
                            {activeView === 'dashboard' && (
                                <Dashboard
                                    timerStatus={timerStatus}
                                    tasks={tasks}
                                    onStartTimer={startTimer}
                                    onPauseTimer={pauseTimer}
                                    onStopTimer={stopTimer}
                                    selectedTaskId={selectedTaskId}
                                    setSelectedTaskId={setSelectedTaskId}
                                    onTasksUpdate={fetchTasks}
                                    todayStats={todayStats}
                                />
                            )}

                            {activeView === 'tasks' && (
                                <TaskManagement
                                    tasks={tasks}
                                    onTasksUpdate={fetchTasks}
                                />
                            )}

                            {activeView === 'categories' && (
                                <CategoryManagement
                                    onCategoriesUpdate={() => {
                                        // 分类更新后可能需要刷新任务列表
                                        fetchTasks();
                                    }}
                                />
                            )}

                            {activeView === 'statistics' && (
                                <Statistics />
                            )}

                            {activeView === 'settings' && (
                                <SettingsComponent />
                            )}

                            {activeView === 'about' && (
                                <About />
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </ThemeProvider>
    );
}

export default App;
