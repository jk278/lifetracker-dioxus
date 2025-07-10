import { invoke } from "@tauri-apps/api/core";
import {
	ArrowLeft,
	ArrowRight,
	Cloud,
	Database,
	Download,
	Info,
	RefreshCw,
	Trash2,
	Upload,
} from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useDataRefresh } from "../../hooks/useDataRefresh";
import { useNavigation } from "../../hooks/useRouter";

export function DataManagement() {
	const { navigate, canGoBack, goBack } = useNavigation();

	// 固定首次进入时是否可返回，避免 goBack 后按钮立刻消失
	const showBackButton = useRef(canGoBack).current;

	// 数据统计状态
	const [statistics, setStatistics] = useState({
		total_tasks: 0,
		total_time_spent: 0,
		total_transactions: 0,
		total_notes: 0,
		database_size: "未知",
		last_backup: "从未",
	});

	const [loading, setLoading] = useState(false);
	const [operationStatus, setOperationStatus] = useState<{
		type: "success" | "error" | null;
		message: string;
	}>({ type: null, message: "" });

	// 获取数据统计信息
	const fetchStatistics = useCallback(async () => {
		setLoading(true);
		try {
			const stats = await invoke<{
				total_tasks: number;
				total_time_spent: number;
				total_transactions: number;
				total_notes: number;
				database_size: string;
				last_backup: string;
			}>("get_data_statistics");
			setStatistics(stats);
		} catch (error) {
			console.error("获取数据统计失败:", error);
			setOperationStatus({
				type: "error",
				message: "获取数据统计失败，请重试。",
			});
		} finally {
			setLoading(false);
		}
	}, []);

	// 设置数据刷新监听 - 监听所有数据变化事件
	useDataRefresh(fetchStatistics, {
		onRefresh: (changeType) => {
			console.log(`数据管理页面收到数据变化通知: ${changeType}`);
			// 数据变化后清除之前的状态消息
			setOperationStatus({ type: null, message: "" });
		},
	});

	// 初始化获取统计信息
	useEffect(() => {
		fetchStatistics();
	}, [fetchStatistics]);

	// 自动清除状态消息
	useEffect(() => {
		if (operationStatus.type) {
			const timer = setTimeout(
				() => setOperationStatus({ type: null, message: "" }),
				5000,
			);
			return () => clearTimeout(timer);
		}
	}, [operationStatus]);

	// 导航到子页面
	const handleNavigateToExport = useCallback(() => {
		navigate("data-export", "system");
	}, [navigate]);

	const handleNavigateToImport = useCallback(() => {
		navigate("data-import", "system");
	}, [navigate]);

	const handleNavigateToBackup = useCallback(() => {
		navigate("data-backup", "system");
	}, [navigate]);

	const handleNavigateToSync = useCallback(() => {
		navigate("data-sync", "system");
	}, [navigate]);

	const handleNavigateToCleanup = useCallback(() => {
		navigate("data-cleanup", "system");
	}, [navigate]);

	// 返回处理
	const handleBack = useCallback(() => {
		if (canGoBack) {
			goBack();
		} else {
			navigate("system", "direct");
		}
	}, [canGoBack, goBack, navigate]);

	// 功能卡片数据
	const features = [
		{
			id: "export",
			icon: Download,
			title: "数据导出",
			description: "导出任务、财务、笔记等数据",
			color: "text-blue-600 dark:text-blue-400",
			bgColor: "bg-blue-50 dark:bg-blue-900/20",
			onClick: handleNavigateToExport,
		},
		{
			id: "import",
			icon: Upload,
			title: "数据导入",
			description: "从备份文件导入数据",
			color: "text-green-600 dark:text-green-400",
			bgColor: "bg-green-50 dark:bg-green-900/20",
			onClick: handleNavigateToImport,
		},
		{
			id: "backup",
			icon: Database,
			title: "备份与恢复",
			description: "创建备份和从备份恢复数据",
			color: "text-purple-600 dark:text-purple-400",
			bgColor: "bg-purple-50 dark:bg-purple-900/20",
			onClick: handleNavigateToBackup,
		},
		{
			id: "sync",
			icon: Cloud,
			title: "多端同步",
			description: "配置 WebDAV 云同步",
			color: "text-indigo-600 dark:text-indigo-400",
			bgColor: "bg-indigo-50 dark:bg-indigo-900/20",
			onClick: handleNavigateToSync,
		},
		{
			id: "cleanup",
			icon: Trash2,
			title: "数据清理",
			description: "永久删除所有数据（危险操作）",
			color: "text-red-600 dark:text-red-400",
			bgColor: "bg-red-50 dark:bg-red-900/20",
			onClick: handleNavigateToCleanup,
		},
	];

	return (
		<div className="h-full flex flex-col">
			{/* 顶部工具栏 */}
			<div className="flex-shrink-0 px-4 md:px-6 py-4 border-b border-gray-200 dark:border-gray-700 surface-adaptive">
				<div className="flex items-center justify-between">
					<div className="flex items-center space-x-3">
						{showBackButton && (
							<button
								onClick={() => {
									if (canGoBack) {
										goBack();
									}
								}}
								className="flex items-center justify-center w-8 h-8 text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
								title="返回"
							>
								<ArrowLeft className="w-5 h-5" />
							</button>
						)}
						<h1 className="text-2xl font-bold text-gray-900 dark:text-white">
							数据管理
						</h1>
					</div>
					<button
						onClick={fetchStatistics}
						className="flex items-center justify-center w-8 h-8 text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
						title="刷新数据"
					>
						<RefreshCw className="w-5 h-5" />
					</button>
				</div>
			</div>

			{/* 可滚动内容区域 */}
			<div className="flex-1 overflow-y-auto px-4 md:px-6 py-6">
				<div className="max-w-4xl mx-auto space-y-6">
					{/* 状态消息 */}
					{operationStatus.type && (
						<div
							className={`p-4 rounded-lg ${
								operationStatus.type === "success"
									? "bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 text-green-700 dark:text-green-300"
									: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-300"
							}`}
						>
							{operationStatus.message}
						</div>
					)}

					{/* 数据统计概览 */}
					<div className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6">
						<div className="flex items-center justify-between mb-4">
							<h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
								数据统计概览
							</h2>
							{loading && (
								<div className="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-600" />
							)}
						</div>
						<div className="grid grid-cols-2 md:grid-cols-3 gap-4">
							<div className="text-center">
								<div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
									{statistics.total_tasks}
								</div>
								<div className="text-sm text-gray-600 dark:text-gray-400">
									任务数量
								</div>
							</div>
							<div className="text-center">
								<div className="text-2xl font-bold text-green-600 dark:text-green-400">
									{Math.round(statistics.total_time_spent / 3600)}h
								</div>
								<div className="text-sm text-gray-600 dark:text-gray-400">
									总时长
								</div>
							</div>
							<div className="text-center">
								<div className="text-2xl font-bold text-purple-600 dark:text-purple-400">
									{statistics.total_transactions}
								</div>
								<div className="text-sm text-gray-600 dark:text-gray-400">
									交易记录
								</div>
							</div>
							<div className="text-center">
								<div className="text-2xl font-bold text-orange-600 dark:text-orange-400">
									{statistics.total_notes}
								</div>
								<div className="text-sm text-gray-600 dark:text-gray-400">
									笔记数量
								</div>
							</div>
							<div className="text-center">
								<div className="text-2xl font-bold text-indigo-600 dark:text-indigo-400">
									{statistics.database_size}
								</div>
								<div className="text-sm text-gray-600 dark:text-gray-400">
									数据库大小
								</div>
							</div>
							<div className="text-center">
								<div className="text-2xl font-bold text-gray-600 dark:text-gray-400">
									{statistics.last_backup}
								</div>
								<div className="text-sm text-gray-600 dark:text-gray-400">
									最后备份
								</div>
							</div>
						</div>
					</div>

					{/* 功能卡片网格 */}
					<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
						{features.map((feature) => (
							<button
								key={feature.id}
								onClick={feature.onClick}
								className="group p-6 surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 hover:shadow-md dark:hover:shadow-lg hover:border-gray-300 dark:hover:border-gray-600 transition-all duration-200 text-left"
							>
								<div className="flex items-center space-x-3 mb-3">
									<div className={`p-2 rounded-lg ${feature.bgColor}`}>
										<feature.icon className={`h-6 w-6 ${feature.color}`} />
									</div>
									<ArrowRight className="h-4 w-4 text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300 transition-colors ml-auto" />
								</div>
								<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
									{feature.title}
								</h3>
								<p className="text-sm text-gray-600 dark:text-gray-400">
									{feature.description}
								</p>
							</button>
						))}
					</div>

					{/* 数据管理说明 */}
					<div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-6">
						<div className="flex items-start">
							<Info className="h-5 w-5 text-blue-600 dark:text-blue-400 mr-3 mt-0.5 flex-shrink-0" />
							<div>
								<h3 className="text-lg font-semibold text-blue-900 dark:text-blue-100 mb-2">
									数据管理说明
								</h3>
								<div className="text-sm text-blue-700 dark:text-blue-300 space-y-1">
									<p>
										• <strong>数据导出</strong>：将数据保存为 JSON、CSV 或 XML
										格式
									</p>
									<p>
										• <strong>数据导入</strong>：从导出的文件中恢复数据
									</p>
									<p>
										• <strong>备份与恢复</strong>：创建完整的数据库备份
									</p>
									<p>
										• <strong>多端同步</strong>：通过 WebDAV 实现设备间数据同步
									</p>
									<p>
										• <strong>数据清理</strong>：永久删除所有数据（请谨慎使用）
									</p>
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
}
