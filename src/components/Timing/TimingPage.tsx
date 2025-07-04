import type React from "react";
import { useState } from "react";
import type { Task, TimerStatus } from "../../types";
import CategoryManagement from "./CategoryManagement";
import Dashboard from "./Dashboard";
import Statistics from "./Statistics";
import TaskManagement from "./TaskManagement";

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

const TimingPage: React.FC<TimingPageProps> = ({
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

	const tabs = [
		{ key: "dashboard", label: "仪表板" },
		{ key: "tasks", label: "任务管理" },
		{ key: "categories", label: "分类管理" },
		{ key: "statistics", label: "统计报告" },
	];

	return (
		<div className="space-y-6 px-2">
			{/* 内部标签导航 */}
			<div className="flex border-b border-gray-200 dark:border-gray-700 overflow-x-auto">
				{tabs.map((tab) => (
					<button
						key={tab.key}
						onClick={() => setActiveTab(tab.key as any)}
						className={`px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap ${
							activeTab === tab.key
								? "text-theme-primary border-theme-primary border-b-2"
								: "text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 border-b-2 border-transparent"
						}`}
					>
						{tab.label}
					</button>
				))}
			</div>

			{/* 对应内容 */}
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

			{activeTab === "tasks" && (
				<TaskManagement tasks={tasks} onTasksUpdate={onTasksUpdate} />
			)}

			{activeTab === "categories" && (
				<CategoryManagement onCategoriesUpdate={onCategoriesUpdate} />
			)}

			{activeTab === "statistics" && <Statistics />}
		</div>
	);
};

export default TimingPage;
