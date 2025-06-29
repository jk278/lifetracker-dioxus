import { invoke } from "@tauri-apps/api/core";
import {
	Activity,
	BarChart3,
	Brain,
	Clock,
	Gauge,
	History,
	Pause,
	Play,
	Plus,
	Square,
	Target,
	TrendingUp,
	X,
} from "lucide-react";
import type React from "react";
import { useCallback, useEffect, useState } from "react";
import type { Category, Task, TimeEntry, TimerStatus } from "../types";

interface DashboardProps {
	timerStatus: TimerStatus;
	tasks: Task[];
	onStartTimer: (taskId: string) => void;
	onPauseTimer: () => void;
	onStopTimer: () => void;
	selectedTaskId: string;
	setSelectedTaskId: (id: string) => void;
	onTasksUpdate: () => void;
	todayStats: {
		totalSeconds: number;
		taskCount: number;
		averageSeconds: number;
		efficiency: number;
		efficiencyDetails: {
			focusScore: number;
			volumeScore: number;
			rhythmScore: number;
			avgSessionMinutes: number;
			hoursWorked: number;
			actualSessionsPerHour: number;
		};
	};
}

// 格式化时间函数
const formatDuration = (seconds: number): string => {
	const hours = Math.floor(seconds / 3600);
	const minutes = Math.floor((seconds % 3600) / 60);
	const secs = seconds % 60;
	return `${hours.toString().padStart(2, "0")}:${minutes.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
};

const Dashboard: React.FC<DashboardProps> = ({
	timerStatus,
	tasks,
	onStartTimer,
	onPauseTimer,
	onStopTimer,
	selectedTaskId,
	setSelectedTaskId,
	onTasksUpdate,
	todayStats,
}) => {
	const [newTaskName, setNewTaskName] = useState("");
	const [newTaskDescription, setNewTaskDescription] = useState("");
	const [categories, setCategories] = useState<Category[]>([]);
	const [selectedCategoryId, setSelectedCategoryId] = useState<string>("");
	const [showQuickStart, setShowQuickStart] = useState(false);
	const [todayTimeEntries, setTodayTimeEntries] = useState<TimeEntry[]>([]);
	const [showEfficiencyDetails, setShowEfficiencyDetails] = useState(false);

	// 获取分类列表
	const fetchCategories = useCallback(async () => {
		try {
			const categoryList = await invoke<Category[]>("get_categories");
			setCategories(categoryList);
		} catch (error) {
			console.error("获取分类列表失败:", error);
		}
	}, []);

	// 获取今日时间记录
	const fetchTodayTimeEntries = useCallback(async () => {
		try {
			const entries = await invoke<any[]>("get_today_time_entries");
			console.log("获取到今日时间记录:", entries);

			// 转换为 TimeEntry 类型
			const formattedEntries: TimeEntry[] = entries.map((entry) => ({
				id: entry.id,
				task_name: entry.task_name,
				start_time: entry.start_time,
				end_time: entry.end_time,
				duration_seconds: entry.duration_seconds,
			}));

			setTodayTimeEntries(formattedEntries);
		} catch (error) {
			console.error("获取今日时间记录失败:", error);
		}
	}, []);

	// 创建新任务
	const createTask = async () => {
		if (!newTaskName.trim()) return;

		try {
			console.log("创建任务开始，参数:", {
				name: newTaskName,
				description: newTaskDescription || null,
				category_id: selectedCategoryId || null,
				tags: null,
			});

			const result = await invoke("create_task", {
				request: {
					name: newTaskName,
					description: newTaskDescription || null,
					category_id: selectedCategoryId || null,
					tags: null,
				},
			});

			console.log("任务创建成功，返回结果:", result);

			setNewTaskName("");
			setNewTaskDescription("");
			setShowQuickStart(false);

			// 稍等一下再刷新，确保数据库操作完全完成
			setTimeout(() => {
				console.log("开始刷新任务列表");
				onTasksUpdate(); // 这会触发父组件中的统计数据更新
				fetchTodayTimeEntries(); // 刷新今日时间记录
			}, 200);
		} catch (error) {
			console.error("创建任务失败:", error);
			alert(`创建任务失败: ${error}`);
		}
	};

	useEffect(() => {
		fetchCategories();
		fetchTodayTimeEntries();
	}, [fetchCategories, fetchTodayTimeEntries]);

	// 监听todayStats变化，当统计数据更新时也更新时间记录
	useEffect(() => {
		fetchTodayTimeEntries();
	}, [todayStats, fetchTodayTimeEntries]);

	return (
		<div className="space-y-6">
			{/* 页面标题 */}
			<div className="flex items-center justify-between">
				<h2 className="text-2xl font-bold text-gray-900 dark:text-white">
					仪表板
				</h2>
				<div className="flex space-x-2">
					<button
						onClick={() => setShowQuickStart(true)}
						className="flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
					>
						<Plus className="h-4 w-4 mr-2" />
						快速开始
					</button>
				</div>
			</div>

			{/* 计时器控制区域 */}
			<div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
				<div className="flex items-center justify-between">
					<div className="flex items-center space-x-6">
						{/* 当前计时器显示 */}
						<div className="text-center">
							<div className="text-4xl font-mono font-bold text-gray-900 dark:text-white mb-2">
								{formatDuration(timerStatus.elapsed_seconds)}
							</div>
							<div className="text-sm text-gray-500 dark:text-gray-400">
								{timerStatus.state === "running"
									? "运行中"
									: timerStatus.state === "paused"
										? "已暂停"
										: "未开始"}
							</div>
						</div>

						{/* 任务选择 */}
						<div className="flex-1 max-w-md">
							<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
								当前任务
							</label>
							<select
								value={selectedTaskId}
								onChange={(e) => setSelectedTaskId(e.target.value)}
								className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
							>
								<option value="">选择任务...</option>
								{tasks.map((task) => (
									<option key={task.id} value={task.id}>
										{task.name}
									</option>
								))}
							</select>
						</div>
					</div>

					{/* 控制按钮 */}
					<div className="flex items-center space-x-3">
						{timerStatus.state === "stopped" ? (
							<button
								onClick={() => selectedTaskId && onStartTimer(selectedTaskId)}
								className="flex items-center px-6 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50"
								disabled={!selectedTaskId}
							>
								<Play className="h-5 w-5 mr-2" />
								开始
							</button>
						) : timerStatus.state === "running" ? (
							<div className="flex space-x-2">
								<button
									onClick={onPauseTimer}
									className="flex items-center px-4 py-3 bg-yellow-600 text-white rounded-lg hover:bg-yellow-700"
								>
									<Pause className="h-5 w-5 mr-2" />
									暂停
								</button>
								<button
									onClick={onStopTimer}
									className="flex items-center px-4 py-3 bg-red-600 text-white rounded-lg hover:bg-red-700"
								>
									<Square className="h-5 w-5 mr-2" />
									停止
								</button>
							</div>
						) : (
							<div className="flex space-x-2">
								<button
									onClick={() => selectedTaskId && onStartTimer(selectedTaskId)}
									className="flex items-center px-4 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700"
								>
									<Play className="h-5 w-5 mr-2" />
									继续
								</button>
								<button
									onClick={onStopTimer}
									className="flex items-center px-4 py-3 bg-red-600 text-white rounded-lg hover:bg-red-700"
								>
									<Square className="h-5 w-5 mr-2" />
									停止
								</button>
							</div>
						)}
					</div>
				</div>
			</div>

			{/* 今日统计卡片 */}
			<div className="grid grid-cols-1 lg:grid-cols-4 md:grid-cols-2 gap-6">
				{/* 今日总时间 */}
				<div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
					<div className="flex items-center">
						<div className="flex-shrink-0">
							<Clock className="h-8 w-8 text-blue-600 dark:text-blue-400" />
						</div>
						<div className="ml-4">
							<p className="text-sm font-medium text-gray-500 dark:text-gray-400">
								今日总时间
							</p>
							<p className="text-2xl font-semibold text-gray-900 dark:text-white">
								{formatDuration(todayStats.totalSeconds)}
							</p>
						</div>
					</div>
				</div>

				{/* 任务数量 */}
				<div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
					<div className="flex items-center">
						<div className="flex-shrink-0">
							<Target className="h-8 w-8 text-green-600 dark:text-green-400" />
						</div>
						<div className="ml-4">
							<p className="text-sm font-medium text-gray-500 dark:text-gray-400">
								今日任务
							</p>
							<p className="text-2xl font-semibold text-gray-900 dark:text-white">
								{todayStats.taskCount}
							</p>
						</div>
					</div>
				</div>

				{/* 平均时间 */}
				<div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
					<div className="flex items-center">
						<div className="flex-shrink-0">
							<BarChart3 className="h-8 w-8 text-purple-600 dark:text-purple-400" />
						</div>
						<div className="ml-4">
							<p className="text-sm font-medium text-gray-500 dark:text-gray-400">
								平均时间
							</p>
							<p className="text-2xl font-semibold text-gray-900 dark:text-white">
								{formatDuration(todayStats.averageSeconds)}
							</p>
						</div>
					</div>
				</div>

				{/* 效率指标 */}
				<div
					className="bg-white dark:bg-gray-800 rounded-lg shadow-lg dark:shadow-gray-700/20 p-6 cursor-pointer hover:shadow-xl dark:hover:shadow-gray-700/30 transition-shadow duration-200"
					onClick={() => setShowEfficiencyDetails(true)}
				>
					<div className="flex items-center">
						<div className="flex-shrink-0">
							<TrendingUp className="h-8 w-8 text-orange-600 dark:text-orange-400" />
						</div>
						<div className="ml-4">
							<p className="text-sm font-medium text-gray-500 dark:text-gray-400">
								效率评分
							</p>
							<p className="text-2xl font-semibold text-gray-900 dark:text-white">
								{todayStats.efficiency}%
							</p>
							<p className="text-xs text-blue-600 dark:text-blue-400 mt-1">
								点击查看详情
							</p>
						</div>
					</div>
				</div>
			</div>

			{/* 今日工作记录 */}
			<div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
				<div className="flex items-center justify-between mb-4">
					<h3 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center">
						<History className="h-5 w-5 mr-2 text-blue-600 dark:text-blue-400" />
						今日工作记录
					</h3>
					<span className="text-sm text-gray-500 dark:text-gray-400">
						共 {todayTimeEntries.length} 条记录
					</span>
				</div>

				{todayTimeEntries.length === 0 ? (
					<div className="text-center py-8 text-gray-500 dark:text-gray-400">
						<History className="h-12 w-12 mx-auto mb-3 opacity-30" />
						<p>今日暂无工作记录</p>
						<p className="text-sm mt-1">开始一个任务来创建记录吧！</p>
					</div>
				) : (
					<div className="space-y-3">
						{todayTimeEntries.map((entry, index) => (
							<div
								key={entry.id}
								className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg border border-gray-200 dark:border-gray-600"
							>
								<div className="flex items-center space-x-4">
									{/* 序号 */}
									<div className="flex-shrink-0 w-8 h-8 bg-blue-100 dark:bg-blue-900 rounded-full flex items-center justify-center">
										<span className="text-sm font-medium text-blue-600 dark:text-blue-400">
											{index + 1}
										</span>
									</div>

									{/* 任务信息 */}
									<div>
										<h4 className="font-medium text-gray-900 dark:text-white">
											{entry.task_name}
										</h4>
										<p className="text-sm text-gray-500 dark:text-gray-400">
											{entry.start_time} - {entry.end_time || "进行中"}
										</p>
									</div>
								</div>

								{/* 时长 */}
								<div className="text-right">
									<div className="font-mono font-medium text-gray-900 dark:text-white">
										{formatDuration(entry.duration_seconds)}
									</div>
									<div className="text-xs text-gray-500 dark:text-gray-400">
										{Math.round(entry.duration_seconds / 60)} 分钟
									</div>
								</div>
							</div>
						))}
					</div>
				)}
			</div>

			{/* 快速开始对话框 */}
			{showQuickStart && (
				<div className="fixed inset-0 bg-black/50 dark:bg-black/70 flex items-center justify-center z-50">
					<div className="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md mx-4 shadow-xl">
						<h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
							创建新任务
						</h3>

						<div className="space-y-4">
							<div>
								<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
									任务名称
								</label>
								<input
									type="text"
									value={newTaskName}
									onChange={(e) => setNewTaskName(e.target.value)}
									className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
									placeholder="输入任务名称..."
									autoFocus
								/>
							</div>

							<div>
								<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
									任务描述
								</label>
								<textarea
									value={newTaskDescription}
									onChange={(e) => setNewTaskDescription(e.target.value)}
									className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
									placeholder="输入任务描述..."
									rows={3}
								/>
							</div>

							<div>
								<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
									分类
								</label>
								<select
									value={selectedCategoryId}
									onChange={(e) => setSelectedCategoryId(e.target.value)}
									className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
								>
									<option value="">无分类</option>
									{categories.map((category) => (
										<option key={category.id} value={category.id}>
											{category.name}
										</option>
									))}
								</select>
							</div>
						</div>

						<div className="flex justify-end space-x-3 mt-6">
							<button
								onClick={() => setShowQuickStart(false)}
								className="px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
							>
								取消
							</button>
							<button
								onClick={createTask}
								className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 transition-colors"
								disabled={!newTaskName.trim()}
							>
								创建
							</button>
						</div>
					</div>
				</div>
			)}

			{/* 效率评分详情弹窗 */}
			{showEfficiencyDetails && (
				<div className="fixed inset-0 bg-black/50 dark:bg-black/70 flex items-center justify-center z-50">
					<div className="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-2xl mx-4 shadow-xl max-h-[90vh] overflow-y-auto">
						{/* 弹窗头部 */}
						<div className="flex items-center justify-between mb-6">
							<h3 className="text-xl font-semibold text-gray-900 dark:text-white flex items-center">
								<TrendingUp className="h-6 w-6 mr-2 text-orange-600 dark:text-orange-400" />
								效率评分详情
							</h3>
							<button
								onClick={() => setShowEfficiencyDetails(false)}
								className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
							>
								<X className="h-5 w-5 text-gray-500 dark:text-gray-400" />
							</button>
						</div>

						{/* 总分展示 */}
						<div className="text-center mb-8">
							<div className="inline-flex items-center justify-center w-20 h-20 bg-gradient-to-br from-orange-400 to-orange-600 rounded-full mb-3">
								<span className="text-2xl font-bold text-white">
									{todayStats.efficiency}
								</span>
							</div>
							<h4 className="text-lg font-medium text-gray-900 dark:text-white">
								今日效率评分
							</h4>
							<p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
								基于专注度、工作量和节奏的综合评估
							</p>
						</div>

						{/* 详细评分项 */}
						<div className="space-y-6">
							{/* 专注度评分 */}
							<div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4">
								<div className="flex items-center justify-between mb-3">
									<div className="flex items-center">
										<Brain className="h-5 w-5 text-blue-600 dark:text-blue-400 mr-2" />
										<h5 className="font-medium text-gray-900 dark:text-white">
											专注度评分
										</h5>
									</div>
									<span className="text-lg font-bold text-blue-600 dark:text-blue-400">
										{todayStats.efficiencyDetails.focusScore}/40
									</span>
								</div>
								<div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 mb-3">
									<div
										className="bg-blue-600 h-2 rounded-full transition-all duration-300"
										style={{
											width: `${(todayStats.efficiencyDetails.focusScore / 40) * 100}%`,
										}}
									/>
								</div>
								<div className="text-sm text-gray-600 dark:text-gray-300 space-y-1">
									<p>
										平均会话时长:{" "}
										<span className="font-medium">
											{todayStats.efficiencyDetails.avgSessionMinutes.toFixed(
												1,
											)}{" "}
											分钟
										</span>
									</p>
									<div className="text-xs text-gray-500 dark:text-gray-400">
										<p>
											• ≥25分钟: 40分 (深度专注) • 15-25分钟: 30分 (良好专注)
										</p>
										<p>
											• 5-15分钟: 20分 (短时专注) • &lt;5分钟: 10分 (过于碎片化)
										</p>
									</div>
								</div>
							</div>

							{/* 工作量评分 */}
							<div className="bg-green-50 dark:bg-green-900/20 rounded-lg p-4">
								<div className="flex items-center justify-between mb-3">
									<div className="flex items-center">
										<Gauge className="h-5 w-5 text-green-600 dark:text-green-400 mr-2" />
										<h5 className="font-medium text-gray-900 dark:text-white">
											工作量评分
										</h5>
									</div>
									<span className="text-lg font-bold text-green-600 dark:text-green-400">
										{todayStats.efficiencyDetails.volumeScore}/30
									</span>
								</div>
								<div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 mb-3">
									<div
										className="bg-green-600 h-2 rounded-full transition-all duration-300"
										style={{
											width: `${(todayStats.efficiencyDetails.volumeScore / 30) * 100}%`,
										}}
									/>
								</div>
								<div className="text-sm text-gray-600 dark:text-gray-300 space-y-1">
									<p>
										今日工作时长:{" "}
										<span className="font-medium">
											{todayStats.efficiencyDetails.hoursWorked.toFixed(1)} 小时
										</span>
									</p>
									<div className="text-xs text-gray-500 dark:text-gray-400">
										<p>
											• ≥6小时: 30分 (饱满) • 4-6小时: 25分 (充实) • 2-4小时:
											20分 (适中)
										</p>
										<p>• 1-2小时: 15分 (轻量) • &lt;1小时: 10分 (起步)</p>
									</div>
								</div>
							</div>

							{/* 节奏评分 */}
							<div className="bg-purple-50 dark:bg-purple-900/20 rounded-lg p-4">
								<div className="flex items-center justify-between mb-3">
									<div className="flex items-center">
										<Activity className="h-5 w-5 text-purple-600 dark:text-purple-400 mr-2" />
										<h5 className="font-medium text-gray-900 dark:text-white">
											节奏评分
										</h5>
									</div>
									<span className="text-lg font-bold text-purple-600 dark:text-purple-400">
										{todayStats.efficiencyDetails.rhythmScore}/30
									</span>
								</div>
								<div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 mb-3">
									<div
										className="bg-purple-600 h-2 rounded-full transition-all duration-300"
										style={{
											width: `${(todayStats.efficiencyDetails.rhythmScore / 30) * 100}%`,
										}}
									/>
								</div>
								<div className="text-sm text-gray-600 dark:text-gray-300 space-y-1">
									{todayStats.efficiencyDetails.hoursWorked >= 0.25 ? (
										<p>
											工作节奏:{" "}
											<span className="font-medium">
												{todayStats.efficiencyDetails.actualSessionsPerHour.toFixed(
													1,
												)}{" "}
												段/小时
											</span>
										</p>
									) : (
										<p>
											工作节奏:{" "}
											<span className="font-medium text-gray-500">
												数据不足 (需≥15分钟)
											</span>
										</p>
									)}
									<div className="text-xs text-gray-500 dark:text-gray-400">
										<p>• 理想节奏: 2段/小时 (每段30分钟)</p>
										<p>• 评分基于实际节奏与理想节奏的匹配度</p>
										{todayStats.efficiencyDetails.hoursWorked < 0.25 && (
											<p>• 工作时间少于15分钟时给予基础分数</p>
										)}
									</div>
								</div>
							</div>
						</div>

						{/* 改进建议 */}
						<div className="mt-6 p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
							<h5 className="font-medium text-gray-900 dark:text-white mb-2">
								💡 改进建议
							</h5>
							<div className="text-sm text-gray-600 dark:text-gray-300 space-y-1">
								{todayStats.efficiencyDetails.focusScore < 30 && (
									<p>• 尝试延长单次工作时间，建议使用番茄工作法(25分钟专注)</p>
								)}
								{todayStats.efficiencyDetails.volumeScore < 20 && (
									<p>• 增加今日工作总时长，保持持续的工作节奏</p>
								)}
								{todayStats.efficiencyDetails.rhythmScore < 20 && (
									<p>• 调整工作节奏，避免过于频繁的开始停止</p>
								)}
								{todayStats.efficiency >= 80 && (
									<p>🎉 效率很高！保持这种良好的工作状态</p>
								)}
							</div>
						</div>

						{/* 关闭按钮 */}
						<div className="flex justify-end mt-6">
							<button
								onClick={() => setShowEfficiencyDetails(false)}
								className="px-4 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700 transition-colors"
							>
								关闭
							</button>
						</div>
					</div>
				</div>
			)}
		</div>
	);
};

export default Dashboard;
