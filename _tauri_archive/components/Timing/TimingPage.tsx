import { memo, useState } from "react";
import type { Task, TimerStatus } from "../../types";
import { TabTransition } from "../Animation";
import CategoryManagement from "./CategoryManagement";
import Dashboard from "./Dashboard";
import Statistics from "./Statistics";
import { TaskManagement } from "./TaskManagement";

interface TimingPageProps {
	timerStatus: TimerStatus;
	tasks: Task[];
	onStartTimer: (taskId: string) => void;
	onPauseTimer: () => void;
	onResumeTimer: () => void;
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
	onCategoriesUpdate: () => void;
}

const TimingPage: React.FC<TimingPageProps> = memo(
	({
		timerStatus,
		tasks,
		onStartTimer,
		onPauseTimer,
		onResumeTimer,
		onStopTimer,
		selectedTaskId,
		setSelectedTaskId,
		onTasksUpdate,
		todayStats,
		onCategoriesUpdate,
	}) => {
		const [activeTab, setActiveTab] = useState<
			"dashboard" | "tasks" | "categories" | "statistics"
		>("dashboard");
		const [previousTab, setPreviousTab] = useState<
			"dashboard" | "tasks" | "categories" | "statistics"
		>("dashboard");

		const tabs = [
			{ key: "dashboard", label: "仪表板" },
			{ key: "tasks", label: "任务管理" },
			{ key: "categories", label: "分类管理" },
			{ key: "statistics", label: "统计报告" },
		];

		return (
			<div className="flex flex-col h-full">
				{/* 内部标签导航 - 固定在顶部 */}
				<div className="flex-shrink-0 surface-adaptive border-b border-gray-200 dark:border-gray-700 overflow-x-auto sticky top-0 z-10 pt-2 md:pt-4">
					<div className="flex px-0 md:px-6">
						{tabs.map((tab) => (
							<div key={tab.key} className="relative">
								<button
									onClick={() => {
										setPreviousTab(activeTab);
										setActiveTab(tab.key as any);
									}}
									className={`px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap ${
										activeTab === tab.key
											? "text-theme-primary"
											: "text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
									}`}
								>
									{tab.label}
								</button>

								{/* 现代化的选中指示器 - 底部细线 */}
								<div
									className={`absolute bottom-0 left-1/2 transform -translate-x-1/2 h-0.5 bg-theme-primary transition-all duration-300 ease-out ${
										activeTab === tab.key ? "w-8 opacity-100" : "w-0 opacity-0"
									}`}
								/>
							</div>
						))}
					</div>
				</div>

				{/* 对应内容 - 可滚动区域 */}
				<div className="flex-1 overflow-y-auto py-4 px-4 md:px-6 scroll-container">
					<TabTransition
						activeKey={activeTab}
						direction="right"
						previousTab={previousTab}
						tabGroup="timing"
					>
						{activeTab === "dashboard" && (
							<Dashboard
								timerStatus={timerStatus}
								tasks={tasks}
								onStartTimer={onStartTimer}
								onPauseTimer={onPauseTimer}
								onResumeTimer={onResumeTimer}
								onStopTimer={onStopTimer}
								selectedTaskId={selectedTaskId}
								setSelectedTaskId={setSelectedTaskId}
								onTasksUpdate={onTasksUpdate}
								todayStats={todayStats}
							/>
						)}

						{activeTab === "tasks" && <TaskManagement />}

						{activeTab === "categories" && (
							<CategoryManagement onCategoriesUpdate={onCategoriesUpdate} />
						)}

						{activeTab === "statistics" && <Statistics />}
					</TabTransition>
				</div>
			</div>
		);
	},
);

export default TimingPage;
