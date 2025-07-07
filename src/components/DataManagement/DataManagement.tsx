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
import { useCallback, useEffect, useState } from "react";
import { useNavigation } from "../../hooks/useRouter";

export function DataManagement() {
	const { navigate, canGoBack, goBack } = useNavigation();
	const isFromSystemPage = canGoBack;

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
		}
	}, [canGoBack, goBack]);

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
			{/* 固定顶部导航栏 */}
			<div className="flex-shrink-0 px-4 md:px-6 py-4 border-b border-gray-200 dark:border-gray-700 surface-adaptive">
				<div className="flex items-center justify-between">
					<div className="flex items-center space-x-3">
						{/* 仅在从系统页面进入时显示返回按钮 */}
						{isFromSystemPage && (
							<button
								onClick={handleBack}
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
				</div>
			</div>

			{/* 可滚动内容区域 */}
			<div className="flex-1 overflow-y-auto py-4 px-4 md:px-6 scroll-container">
				<div className="max-w-4xl mx-auto space-y-6">
					{/* 状态消息 */}
					{operationStatus.type && (
						<div
							className={`p-4 rounded-lg border ${
								operationStatus.type === "success"
									? "bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800 text-green-800 dark:text-green-200"
									: "bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800 text-red-800 dark:text-red-200"
							}`}
						>
							<div className="flex justify-between items-center">
								<span>{operationStatus.message}</span>
								<button
									onClick={() =>
										setOperationStatus({ type: null, message: "" })
									}
									className="text-sm underline hover:no-underline"
								>
									关闭
								</button>
							</div>
						</div>
					)}

					{/* 数据统计概览 */}
					<div className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6">
						<div className="flex items-center mb-4">
							<Info className="h-5 w-5 text-blue-600 dark:text-blue-400 mr-2" />
							<h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100">
								数据统计概览
							</h2>
							<button
								onClick={fetchStatistics}
								className="ml-auto p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
								title="刷新统计数据"
							>
								<RefreshCw className="h-4 w-4" />
							</button>
						</div>

						{loading ? (
							<div className="flex items-center justify-center py-8">
								<div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
								<span className="ml-3 text-gray-600 dark:text-gray-400">
									加载中...
								</span>
							</div>
						) : (
							<div className="grid grid-cols-2 md:grid-cols-3 gap-4">
								<div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
									<div className="text-sm text-gray-600 dark:text-gray-400">
										任务总数
									</div>
									<div className="text-2xl font-bold text-gray-900 dark:text-gray-100">
										{statistics.total_tasks}
									</div>
								</div>

								<div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
									<div className="text-sm text-gray-600 dark:text-gray-400">
										累计时长
									</div>
									<div className="text-2xl font-bold text-gray-900 dark:text-gray-100">
										{Math.floor(statistics.total_time_spent / 3600)}h{" "}
										{Math.floor((statistics.total_time_spent % 3600) / 60)}m
									</div>
								</div>

								<div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
									<div className="text-sm text-gray-600 dark:text-gray-400">
										财务记录
									</div>
									<div className="text-2xl font-bold text-gray-900 dark:text-gray-100">
										{statistics.total_transactions}
									</div>
								</div>

								<div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
									<div className="text-sm text-gray-600 dark:text-gray-400">
										日记数量
									</div>
									<div className="text-2xl font-bold text-gray-900 dark:text-gray-100">
										{statistics.total_notes}
									</div>
								</div>

								<div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
									<div className="text-sm text-gray-600 dark:text-gray-400">
										数据库大小
									</div>
									<div className="text-2xl font-bold text-gray-900 dark:text-gray-100">
										{statistics.database_size}
									</div>
								</div>

								<div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
									<div className="text-sm text-gray-600 dark:text-gray-400">
										最后备份
									</div>
									<div className="text-2xl font-bold text-gray-900 dark:text-gray-100">
										{statistics.last_backup}
									</div>
								</div>
							</div>
						)}
					</div>

					{/* 功能选项卡片 */}
					<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
						{features.map((feature) => {
							const Icon = feature.icon;
							return (
								<button
									key={feature.id}
									onClick={feature.onClick}
									className={`${feature.bgColor} border border-gray-200 dark:border-gray-700 rounded-lg p-6 text-left hover:bg-opacity-80 transition-all duration-200 hover:shadow-md group`}
								>
									<div className="flex items-start justify-between">
										<div>
											<div className="flex items-center mb-2">
												<Icon className={`h-6 w-6 ${feature.color} mr-3`} />
												<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
													{feature.title}
												</h3>
											</div>
											<p className="text-sm text-gray-600 dark:text-gray-400">
												{feature.description}
											</p>
										</div>
										<ArrowRight className="h-5 w-5 text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300 transition-colors" />
									</div>
								</button>
							);
						})}
					</div>

					{/* 使用提示 */}
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
