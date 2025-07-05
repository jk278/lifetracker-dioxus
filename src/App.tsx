import { invoke } from "@tauri-apps/api/core";
import {
	Clock,
	Database,
	Info,
	Menu,
	Pause,
	Play,
	Settings,
	Square,
	Wallet,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import About from "./components/About";
import AccountingPage from "./components/Accounting/AccountingPage";
import { DataManagement } from "./components/DataManagement";
import { ErrorBoundary } from "./components/ErrorBoundary";
import SettingsComponent from "./components/Settings";
import TimingPage from "./components/Timing/TimingPage";
import TitleBar from "./components/TitleBar";
import { useScrollbarHiding } from "./hooks/useScrollbarHiding";
import { ThemeProvider } from "./hooks/useTheme";
import type { Task, TimerStatus } from "./types";

// 格式化时间函数
const formatDuration = (seconds: number): string => {
	const hours = Math.floor(seconds / 3600);
	const minutes = Math.floor((seconds % 3600) / 60);
	const secs = seconds % 60;
	return `${hours.toString().padStart(2, "0")}:${minutes.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
};

// 导航菜单项配置
const NAV_ITEMS = [
	{ id: "timing", name: "计时", icon: Clock },
	{ id: "accounting", name: "记账", icon: Wallet },
	{ id: "data", name: "数据", icon: Database },
	{ id: "settings", name: "设置", icon: Settings },
	{ id: "about", name: "关于", icon: Info },
] as const;

function App() {
	const [timerStatus, setTimerStatus] = useState<TimerStatus>({
		state: "stopped",
		elapsed_seconds: 0,
		total_today_seconds: 0,
	});

	const [tasks, setTasks] = useState<Task[]>([]);
	const [activeView, setActiveView] = useState<
		"timing" | "settings" | "about" | "accounting" | "data"
	>("timing");
	const [selectedTaskId, setSelectedTaskId] = useState<string>("");

	// 侧边栏状态管理 - 简化状态
	const [isCollapsed, setIsCollapsed] = useState<boolean>(false);

	// 窗口宽度状态管理
	const [isMobileLayout, setIsMobileLayout] = useState<boolean>(false);

	// 悬浮按钮动画状态
	const [delayedIconState, setDelayedIconState] = useState<
		"stopped" | "running" | "paused"
	>("stopped");

	const navRef = useScrollbarHiding<HTMLElement>();
	const mainContentRef = useScrollbarHiding<HTMLDivElement>();

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
			// 1. 获取今日已记录的总时长（秒）
			const timerStats = await invoke<TimerStatus>("get_today_stats");
			console.log("后端今日统计数据:", timerStats);

			const totalSeconds = timerStats.total_today_seconds;

			// 2. 获取今日的时间记录，用条目数量作为"任务/会话数量"
			const todayEntries = await invoke<any[]>("get_today_time_entries");
			const taskCount = todayEntries.length;

			// 3. 计算平均时长（秒）
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

	// 恢复计时
	const resumeTimer = useCallback(async () => {
		try {
			const status = await invoke<TimerStatus>("resume_timer");
			setTimerStatus(status);
		} catch (error) {
			console.error("恢复计时失败:", error);
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
			}, 200); // 200ms 后切换图标，比动画稍快结束
			return () => clearTimeout(timer);
		}
		// 开始/暂停状态：立即切换图标，保持位置连续性
		setDelayedIconState(timerStatus.state);
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
			// 从暂停状态恢复运行 - 调用resumeTimer
			resumeTimer();
		}
	}, [pauseTimer, resumeTimer, startTimer, timerStatus.state]);

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
			return "bg-theme-primary bg-theme-primary-hover text-white theme-transition";
		}
		if (timerStatus.state === "running") {
			return "bg-red-600 hover:bg-red-700 text-white theme-transition";
		}
		return "bg-green-600 hover:bg-green-700 text-white theme-transition";
	}, [timerStatus.state]);

	// 判断是否应该显示为展开状态（长方形）
	const isFloatingButtonExpanded = timerStatus.state !== "stopped";

	// 错误处理回调
	const handleError = useCallback(
		(error: Error, errorInfo: React.ErrorInfo) => {
			console.error("App错误边界捕获到错误:", error, errorInfo);
		},
		[],
	);

	// 窗口宽度检测
	useEffect(() => {
		const handleResize = () => {
			const width = window.innerWidth;
			setIsMobileLayout(width < 768); // 768px 作为断点
		};

		// 初始化
		handleResize();

		// 监听窗口大小变化
		window.addEventListener("resize", handleResize);
		return () => window.removeEventListener("resize", handleResize);
	}, []);

	// 移动端模式下强制收起侧边栏
	useEffect(() => {
		if (isMobileLayout) {
			setIsCollapsed(true);
		}
	}, [isMobileLayout]);

	return (
		<ErrorBoundary
			onError={handleError}
			resetKeys={[activeView, isCollapsed ? "collapsed" : "expanded"]}
			resetOnPropsChange={true}
		>
			<ThemeProvider>
				<div className="h-screen w-screen bg-adaptive flex flex-col overflow-hidden performance-optimized">
					{/* 自定义标题栏 */}
					<TitleBar />

					{/* 主要内容区域 */}
					<div className="flex flex-1 overflow-hidden">
						{/* 侧边栏 - 桌面端显示 */}
						{!isMobileLayout && (
							<div
								className={`${
									isCollapsed ? "w-16" : "w-48"
								} surface-adaptive shadow-sm border-r border-gray-200 dark:border-gray-700 flex-shrink-0 transition-all duration-300 ease-out h-full flex flex-col relative overflow-hidden`}
							>
								{/* 折叠展开控制按钮 */}
								<div className="p-2">
									<div className="relative h-12">
										{/* 固定的图标层 */}
										<div className="absolute left-2 top-0 w-8 h-12 flex items-center justify-center z-10">
											<button
												onClick={() => setIsCollapsed(!isCollapsed)}
												className="w-8 h-8 flex items-center justify-center rounded-md transition-colors duration-200 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-600"
												title={isCollapsed ? "展开侧边栏" : "折叠侧边栏"}
											>
												<Menu className="h-5 w-5" />
											</button>
										</div>

										{/* 文本层 - 独立动画 */}
										<div
											className={`absolute left-12 top-0 h-12 flex items-center transition-all duration-300 ease-out ${
												isCollapsed
													? "opacity-0 translate-x-[-8px] pointer-events-none"
													: "opacity-100 translate-x-0"
											}`}
										>
											<span className="text-sm font-medium text-gray-600 dark:text-gray-300 whitespace-nowrap">
												菜单
											</span>
										</div>
									</div>
								</div>

								{/* 导航菜单 */}
								<nav
									ref={navRef}
									className="flex-1 overflow-y-auto scroll-container"
								>
									<div className="p-2 space-y-1">
										{NAV_ITEMS.map(({ id, name, icon: Icon }) => (
											<div key={id} className="relative h-12">
												{/* 背景层 - 完整宽度的点击区域 */}
												<button
													onClick={() => setActiveView(id as any)}
													className={`absolute inset-0 w-full h-12 rounded-lg transition-all duration-200 ${
														activeView === id
															? "bg-theme-primary/10 hover:bg-theme-primary/15"
															: "hover:bg-gray-50 dark:hover:bg-gray-700"
													}`}
													title={isCollapsed ? name : undefined}
												/>

												{/* 固定的图标层 */}
												<div className="absolute left-2 top-0 w-8 h-12 flex items-center justify-center z-10 pointer-events-none">
													<Icon
														size={20}
														className={`transition-colors duration-200 ${
															activeView === id
																? "text-theme-primary"
																: "text-gray-500 dark:text-gray-400"
														}`}
													/>
												</div>

												{/* 文本层 - 独立动画 */}
												<div
													className={`absolute left-12 top-0 h-12 flex items-center transition-all duration-300 ease-out pointer-events-none ${
														isCollapsed
															? "opacity-0 translate-x-[-8px]"
															: "opacity-100 translate-x-0"
													}`}
												>
													<span
														className={`text-sm font-medium whitespace-nowrap transition-colors duration-200 ${
															activeView === id
																? "text-theme-primary"
																: "text-gray-700 dark:text-gray-200"
														}`}
													>
														{name}
													</span>
												</div>
											</div>
										))}
									</div>
								</nav>
							</div>
						)}

						{/* 主内容区 - 简化为直接使用 Tailwind 类 */}
						<div
							ref={mainContentRef}
							className={`flex-1 overflow-y-auto bg-adaptive relative scroll-container ${
								isMobileLayout ? "pb-20" : "" // 移动端底部留出底部菜单栏空间
							}`}
						>
							<div className={`${isMobileLayout ? "p-4" : "p-6"}`}>
								<ErrorBoundary resetKeys={[activeView]}>
									{activeView === "timing" && (
										<TimingPage
											timerStatus={timerStatus}
											tasks={tasks}
											onStartTimer={startTimer}
											onPauseTimer={pauseTimer}
											onResumeTimer={resumeTimer}
											onStopTimer={stopTimer}
											selectedTaskId={selectedTaskId}
											setSelectedTaskId={setSelectedTaskId}
											onTasksUpdate={fetchTasks}
											todayStats={todayStats}
											onCategoriesUpdate={() => fetchTasks()}
										/>
									)}

									{activeView === "settings" && <SettingsComponent />}

									{activeView === "about" && <About />}

									{activeView === "accounting" && <AccountingPage />}

									{activeView === "data" && <DataManagement />}
								</ErrorBoundary>
							</div>
						</div>
					</div>

					{/* 底部菜单栏 - 移动端显示 */}
					{isMobileLayout && (
						<div className="fixed bottom-0 left-0 right-0 h-16 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 z-40">
							<div className="flex h-full">
								{NAV_ITEMS.map(({ id, name, icon: Icon }) => (
									<button
										key={id}
										onClick={() => setActiveView(id as any)}
										className={`flex-1 flex flex-col items-center justify-center space-y-1 transition-colors duration-200 ${
											activeView === id
												? "text-theme-primary bg-theme-primary/5"
												: "text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700"
										}`}
									>
										<Icon size={20} />
										<span className="text-xs font-medium">{name}</span>
									</button>
								))}
							</div>
						</div>
					)}

					{/* 悬浮按钮 - 仅在仪表板(计时页面)显示 */}
					{activeView === "timing" && (
						<div
							className={`fixed right-6 z-40 ${
								isMobileLayout ? "bottom-20" : "bottom-6" // 移动端避开底部菜单栏
							}`}
						>
							<div
								className={`${getFloatingButtonStyle()} shadow-lg flex items-center rounded-lg relative transition-all duration-300 ease-out ${
									isFloatingButtonExpanded
										? "w-52 h-14" // 展开状态：长方形
										: "w-14 h-14" // 收缩状态：正方形
								}`}
							>
								{/* 左侧内容区域 - 简化过渡动画 */}
								<div
									className={`flex items-center space-x-3 pl-4 overflow-hidden transition-all duration-300 ease-out ${
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
					)}
				</div>
			</ThemeProvider>
		</ErrorBoundary>
	);
}

export default App;
