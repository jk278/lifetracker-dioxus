import { invoke } from "@tauri-apps/api/core";
import {
	BarChart3,
	Calendar,
	ChevronLeft,
	ChevronRight,
	Clock,
	Folder,
	Info,
	Pause,
	Play,
	Settings,
	Square,
} from "lucide-react";
import type React from "react";
import { useCallback, useEffect, useState } from "react";
import About from "./components/About";
import CategoryManagement from "./components/CategoryManagement";
import Dashboard from "./components/Dashboard";
import SettingsComponent from "./components/Settings";
import Statistics from "./components/Statistics";
import TaskManagement from "./components/TaskManagement";
import { ThemeProvider } from "./hooks/useTheme";
import type { Task, TimerStatus } from "./types";

// 格式化时间函数
const formatDuration = (seconds: number): string => {
	const hours = Math.floor(seconds / 3600);
	const minutes = Math.floor((seconds % 3600) / 60);
	const secs = seconds % 60;
	return `${hours.toString().padStart(2, "0")}:${minutes.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
};

function App() {
	const [timerStatus, setTimerStatus] = useState<TimerStatus>({
		state: "stopped",
		elapsed_seconds: 0,
		total_today_seconds: 0,
	});

	const [tasks, setTasks] = useState<Task[]>([]);
	const [activeView, setActiveView] = useState<
		"dashboard" | "tasks" | "categories" | "statistics" | "settings" | "about"
	>("dashboard");
	const [selectedTaskId, setSelectedTaskId] = useState<string>("");

	// 侧边栏状态管理
	const [sidebarWidth, setSidebarWidth] = useState<number>(256); // 256px 默认宽度
	const [isCollapsed, setIsCollapsed] = useState<boolean>(false);
	const [isDragging, setIsDragging] = useState<boolean>(false);

	// 悬浮按钮动画状态
	const [isAnimating, setIsAnimating] = useState<boolean>(false);
	const [delayedIconState, setDelayedIconState] = useState<
		"stopped" | "running" | "paused"
	>("stopped");

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
			actualSessionsPerHour: 0,
		},
	});

	// 获取计时器状态
	const fetchTimerStatus = useCallback(async () => {
		try {
			const status = await invoke<TimerStatus>("get_timer_status");
			setTimerStatus(status);
		} catch (error) {
			console.error("获取计时器状态失败:", error);
		}
	}, []);

	// 获取任务列表
	const fetchTasks = useCallback(async () => {
		try {
			const taskList = await invoke<Task[]>("get_tasks");
			setTasks(taskList);
			if (taskList.length > 0 && !selectedTaskId) {
				setSelectedTaskId(taskList[0].id);
			}
		} catch (error) {
			console.error("获取任务列表失败:", error);
		}
	}, [selectedTaskId]);

	// 获取今日统计数据
	const fetchTodayStats = useCallback(async () => {
		try {
			// 从后端获取今日真实统计数据
			const todayStats = await invoke<TimerStatus>("get_today_stats");
			console.log("后端今日统计数据:", todayStats);

			const totalSeconds = todayStats.total_today_seconds;
			const taskCount = todayStats.elapsed_seconds; // 复用这个字段传递任务数
			const averageSeconds =
				taskCount > 0 ? Math.round(totalSeconds / taskCount) : 0;

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
				avgSessionMinutes = totalSeconds / 60 / taskCount; // 平均每段工作时长（分钟）

				// 1. 专注度评分 (40分) - 基于平均会话时长
				if (avgSessionMinutes >= 25)
					focusScore = 40; // 25分钟以上 = 专注
				else if (avgSessionMinutes >= 15)
					focusScore = 30; // 15-25分钟 = 良好
				else if (avgSessionMinutes >= 5)
					focusScore = 20; // 5-15分钟 = 一般
				else focusScore = 10; // 5分钟以下 = 需改进

				// 2. 工作量评分 (30分) - 基于总工作时长
				if (hoursWorked >= 6)
					volumeScore = 30; // 6小时以上 = 饱满
				else if (hoursWorked >= 4)
					volumeScore = 25; // 4-6小时 = 充实
				else if (hoursWorked >= 2)
					volumeScore = 20; // 2-4小时 = 适中
				else if (hoursWorked >= 1)
					volumeScore = 15; // 1-2小时 = 轻量
				else volumeScore = 10; // 1小时以下 = 起步

				// 3. 节奏评分 (30分) - 基于工作段数与时长的平衡
				const idealSessionsPerHour = 2; // 理想：每小时2段（30分钟一段）

				// 当工作时间少于15分钟时，不计算节奏评分，避免误导性数字
				if (hoursWorked >= 0.25) {
					// 至少15分钟
					actualSessionsPerHour = taskCount / hoursWorked;
					const rhythmRatio = Math.min(
						actualSessionsPerHour / idealSessionsPerHour,
						idealSessionsPerHour / actualSessionsPerHour,
					);
					rhythmScore = Math.round(30 * rhythmRatio);
				} else {
					// 工作时间太短，按基础分给分
					actualSessionsPerHour = 0;
					rhythmScore = 15; // 给予基础分数
				}

				efficiency = Math.min(focusScore + volumeScore + rhythmScore, 100);

				console.log("效率评分详情:", {
					avgSessionMinutes: avgSessionMinutes.toFixed(1),
					focusScore,
					volumeScore,
					rhythmScore,
					actualSessionsPerHour: actualSessionsPerHour.toFixed(1),
					finalEfficiency: efficiency,
				});
			}

			console.log("最终统计数据:", {
				totalSeconds,
				taskCount,
				averageSeconds,
				efficiency,
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
					actualSessionsPerHour,
				},
			});
		} catch (error) {
			console.error("获取统计数据失败:", error);
		}
	}, []);

	// 开始计时
	const startTimer = useCallback(
		async (taskId?: string) => {
			const targetTaskId = taskId || selectedTaskId;
			if (!targetTaskId) return;
			try {
				const status = await invoke<TimerStatus>("start_timer", {
					taskId: targetTaskId,
				});
				setTimerStatus(status);
				if (taskId) setSelectedTaskId(taskId);
			} catch (error) {
				console.error("开始计时失败:", error);
			}
		},
		[selectedTaskId],
	);

	// 暂停计时
	const pauseTimer = useCallback(async () => {
		try {
			const status = await invoke<TimerStatus>("pause_timer");
			setTimerStatus(status);
		} catch (error) {
			console.error("暂停计时失败:", error);
		}
	}, []);

	// 停止计时
	const stopTimer = useCallback(async () => {
		try {
			const status = await invoke<TimerStatus>("stop_timer");
			setTimerStatus(status);
			await fetchTasks(); // 刷新任务列表
			await fetchTimerStatus(); // 重新获取计时器状态以更新今日总时间
			await fetchTodayStats(); // 任务停止后更新统计数据
		} catch (error) {
			console.error("停止计时失败:", error);
		}
	}, [fetchTasks, fetchTimerStatus, fetchTodayStats]);

	// 侧边栏拖拽处理
	const minSidebarWidth = 60; // 最小宽度（折叠状态）
	const maxSidebarWidth = 400; // 最大宽度
	const minMainContentWidth = 300; // 主内容区最小宽度

	// 计算最优侧边栏宽度
	const getOptimalSidebarWidth = useCallback(() => {
		const windowWidth = window.innerWidth;
		const availableWidth = windowWidth - minMainContentWidth;
		const optimalWidth = Math.min(
			Math.max(256, availableWidth * 0.25),
			maxSidebarWidth,
		);
		return Math.max(optimalWidth, minSidebarWidth);
	}, []);

	// 响应式调整
	useEffect(() => {
		const handleResize = () => {
			if (!isDragging && !isCollapsed) {
				const newWidth = getOptimalSidebarWidth();
				setSidebarWidth(newWidth);
			}
		};

		window.addEventListener("resize", handleResize);
		return () => window.removeEventListener("resize", handleResize);
	}, [isDragging, isCollapsed, getOptimalSidebarWidth]);

	// 鼠标拖拽处理
	const handleMouseDown = (e: React.MouseEvent) => {
		e.preventDefault();
		setIsDragging(true);
	};

	const handleMouseMove = useCallback(
		(e: MouseEvent) => {
			if (!isDragging) return;

			const newWidth = e.clientX;
			const windowWidth = window.innerWidth;
			const remainingWidth = windowWidth - newWidth;

			// 确保主内容区和侧边栏都有足够的宽度
			if (
				newWidth >= minSidebarWidth &&
				remainingWidth >= minMainContentWidth &&
				newWidth <= maxSidebarWidth
			) {
				setSidebarWidth(newWidth);

				// 如果拖拽到很小的宽度，自动折叠
				if (newWidth <= minSidebarWidth + 20) {
					setIsCollapsed(true);
					setSidebarWidth(minSidebarWidth);
				} else if (isCollapsed && newWidth > minSidebarWidth + 50) {
					setIsCollapsed(false);
				}
			}
		},
		[isDragging, isCollapsed],
	);

	const handleMouseUp = useCallback(() => {
		setIsDragging(false);
	}, []);

	const toggleSidebar = useCallback(() => {
		if (isCollapsed) {
			setIsCollapsed(false);
			setSidebarWidth(getOptimalSidebarWidth());
		} else {
			setIsCollapsed(true);
			setSidebarWidth(minSidebarWidth);
		}
	}, [isCollapsed, getOptimalSidebarWidth]);

	const handleDoubleClick = useCallback(() => {
		setSidebarWidth(getOptimalSidebarWidth());
		setIsCollapsed(false);
	}, [getOptimalSidebarWidth]);

	// 拖拽事件监听
	useEffect(() => {
		if (isDragging) {
			document.addEventListener("mousemove", handleMouseMove);
			document.addEventListener("mouseup", handleMouseUp);

			return () => {
				document.removeEventListener("mousemove", handleMouseMove);
				document.removeEventListener("mouseup", handleMouseUp);
			};
		}
	}, [isDragging, handleMouseMove, handleMouseUp]);

	// 创建任务功能已移到各个组件中

	// 初始化数据
	useEffect(() => {
		fetchTimerStatus();
		fetchTasks();
	}, [fetchTasks, fetchTimerStatus]);

	// 只在任务列表变化时更新统计数据（不包括计时器状态变化）
	useEffect(() => {
		fetchTodayStats();
	}, [fetchTodayStats]);

	// 定时器更新效果
	useEffect(() => {
		let interval: number | null = null;

		if (timerStatus.state === "running") {
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
	}, [timerStatus.state, fetchTimerStatus]);

	// 处理图标状态的延迟切换
	useEffect(() => {
		if (timerStatus.state === "stopped") {
			// 停止状态：延迟切换图标，让收缩动画先完成大部分
			const timer = setTimeout(() => {
				setDelayedIconState("stopped");
				setIsAnimating(false);
			}, 200); // 200ms 后切换图标，比动画稍快结束
			setIsAnimating(true);
			return () => clearTimeout(timer);
		}
		// 开始/暂停状态：立即切换图标，保持位置连续性
		setDelayedIconState(timerStatus.state);
		setIsAnimating(false);
	}, [timerStatus.state]);

	// 初始化延迟图标状态
	useEffect(() => {
		setDelayedIconState(timerStatus.state);
	}, [timerStatus.state]);

	// 点击悬浮按钮时的处理
	const handleFloatingButtonClick = useCallback(() => {
		if (timerStatus.state === "stopped") {
			// 停止状态直接开始计时
			startTimer();
		} else if (timerStatus.state === "running") {
			pauseTimer();
		} else if (timerStatus.state === "paused") {
			// 从暂停状态恢复运行 - 调用pauseTimer实际上会恢复运行
			pauseTimer();
		}
	}, [pauseTimer, startTimer, timerStatus.state]);

	// 获取悬浮按钮的图标 - 使用延迟状态实现平滑切换
	const getFloatingButtonIcon = useCallback(() => {
		if (delayedIconState === "stopped") {
			return <Play className="h-5 w-5" />;
		}
		if (delayedIconState === "running") {
			return <Pause className="h-5 w-5" />;
		}
		return <Play className="h-5 w-5" />;
	}, [delayedIconState]);

	// 获取悬浮按钮的样式
	const getFloatingButtonStyle = useCallback(() => {
		if (timerStatus.state === "stopped") {
			return "bg-blue-600 hover:bg-blue-700 text-white";
		}
		if (timerStatus.state === "running") {
			return "bg-red-600 hover:bg-red-700 text-white";
		}
		return "bg-green-600 hover:bg-green-700 text-white";
	}, [timerStatus.state]);

	// 判断是否应该显示为展开状态（长方形）
	const isFloatingButtonExpanded = timerStatus.state !== "stopped";

	return (
		<ThemeProvider>
			<div className="h-screen w-screen bg-gray-50 dark:bg-gray-900 transition-colors flex overflow-hidden">
				{/* 侧边栏 - 可调整宽度 */}
				<div
					className={`bg-white dark:bg-gray-800 shadow-sm border-r border-gray-200 dark:border-gray-700 flex-shrink-0 relative h-full flex flex-col ${
						isDragging ? "" : "transition-all duration-200 ease-out"
					}`}
					style={{ width: `${sidebarWidth}px` }}
				>
					{/* 侧边栏头部 - 折叠/展开按钮 */}
					<div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700 flex-shrink-0">
						{!isCollapsed && (
							<h2 className="text-lg font-semibold text-gray-900 dark:text-white">
								导航
							</h2>
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
					<nav className="flex-1 overflow-y-auto">
						<div className={`p-4 space-y-2 ${isCollapsed ? "px-2" : ""}`}>
							{[
								{ id: "dashboard", name: "仪表板", icon: BarChart3 },
								{ id: "tasks", name: "任务管理", icon: Clock },
								{ id: "categories", name: "分类管理", icon: Folder },
								{ id: "statistics", name: "统计报告", icon: Calendar },
								{ id: "settings", name: "设置", icon: Settings },
								{ id: "about", name: "关于", icon: Info },
							].map(({ id, name, icon: Icon }) => (
								<button
									key={id}
									onClick={() => setActiveView(id as any)}
									className={`w-full flex items-center ${isCollapsed ? "justify-center px-2" : "px-4"} py-3 text-sm font-medium rounded-lg transition-all duration-200 ${
										activeView === id
											? "bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300"
											: "text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700"
									}`}
									title={isCollapsed ? name : undefined}
								>
									<Icon className={`h-5 w-5 ${!isCollapsed ? "mr-3" : ""}`} />
									{!isCollapsed && <span className="truncate">{name}</span>}
								</button>
							))}
						</div>
					</nav>

					{/* 拖拽调整手柄 */}
					<div
						className={`group absolute top-0 right-0 w-1 h-full cursor-col-resize transition-all duration-150 ${
							isDragging
								? "bg-blue-500 shadow-lg"
								: "bg-transparent hover:bg-blue-400 hover:shadow-md"
						}`}
						onMouseDown={handleMouseDown}
						onDoubleClick={handleDoubleClick}
						title="拖拽调整宽度 | 双击重置"
					>
						{/* 扩大点击区域 */}
						<div className="absolute inset-y-0 -right-2 w-5 h-full" />
						{/* 视觉指示器 */}
						<div
							className={`absolute top-1/2 -translate-y-1/2 -right-0.5 w-2 h-8 rounded-full transition-all duration-150 ${
								isDragging
									? "bg-blue-600 opacity-100"
									: "bg-gray-400 dark:bg-gray-500 opacity-0 group-hover:opacity-60"
							}`}
						/>
					</div>
				</div>

				{/* 主内容区 - 可滚动 */}
				<div
					className="flex-1 overflow-y-auto bg-gray-50 dark:bg-gray-900 relative"
					style={{ minWidth: `${minMainContentWidth}px` }}
				>
					<div className="p-8">
						{activeView === "dashboard" && (
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

						{activeView === "tasks" && (
							<TaskManagement tasks={tasks} onTasksUpdate={fetchTasks} />
						)}

						{activeView === "categories" && (
							<CategoryManagement
								onCategoriesUpdate={() => {
									// 分类更新后可能需要刷新任务列表
									fetchTasks();
								}}
							/>
						)}

						{activeView === "statistics" && <Statistics />}

						{activeView === "settings" && <SettingsComponent />}

						{activeView === "about" && <About />}
					</div>

					{/* 悬浮按钮 - 右下角 */}
					<div className="fixed bottom-6 right-6 z-50">
						<div
							className={`${getFloatingButtonStyle()} shadow-lg transition-all duration-300 ease-in-out flex items-center rounded-lg relative ${
								isFloatingButtonExpanded
									? "w-52 h-14" // 展开状态：长方形
									: "w-14 h-14" // 收缩状态：正方形
							}`}
						>
							{/* 左侧内容区域 - 从右侧逐渐展开，避免从边缘突然出现 */}
							<div
								className={`flex items-center space-x-3 pl-4 transition-all duration-300 ease-in-out overflow-hidden ${
									isFloatingButtonExpanded
										? "w-40 opacity-100 translate-x-0" // 展开：显示内容，从右侧滑入
										: "w-0 opacity-0 translate-x-4" // 收缩：隐藏内容，向右侧滑出
								}`}
							>
								{/* 固定宽度的时间显示区域 */}
								<div className="text-left w-24 flex-shrink-0">
									<div className="text-base font-mono font-bold text-white leading-tight">
										{formatDuration(timerStatus.elapsed_seconds)}
									</div>
									<div className="text-xs text-white/80 leading-tight">
										今日: {formatDuration(timerStatus.total_today_seconds)}
									</div>
								</div>

								{/* 停止按钮 */}
								<button
									onClick={(e) => {
										e.stopPropagation(); // 阻止事件冒泡
										stopTimer();
									}}
									className="p-1.5 bg-white/20 hover:bg-white/30 rounded-md transition-colors flex-shrink-0"
									title="停止计时"
								>
									<Square className="h-4 w-4 text-white" />
								</button>
							</div>

							{/* 主控制按钮 - 绝对定位，始终在右侧固定位置 */}
							<button
								onClick={handleFloatingButtonClick}
								className="absolute right-2 w-10 h-10 flex items-center justify-center rounded-lg transition-all duration-200 hover:bg-white/10"
								title={
									delayedIconState === "stopped"
										? "开始计时"
										: delayedIconState === "running"
											? "暂停计时"
											: "继续计时"
								}
							>
								<div className="transition-transform duration-200">
									{getFloatingButtonIcon()}
								</div>
							</button>
						</div>
					</div>
				</div>
			</div>
		</ThemeProvider>
	);
}

export default App;
