import { invoke } from "@tauri-apps/api/core";
import { appDataDir, join, resolve as pathResolve } from "@tauri-apps/api/path";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
	AlertCircle,
	ArrowLeft,
	Calendar,
	CheckCircle,
	Cloud,
	Download,
	FileText,
	Info,
	RefreshCw,
	Settings,
	Upload,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useRouter } from "../hooks/useRouter";

interface ExportOptions {
	include_categories?: boolean;
	include_statistics?: boolean;
	start_date?: string;
	end_date?: string;
	category_filter?: string[];
	group_by_date?: boolean;
	group_by_category?: boolean;
	include_metadata?: boolean;
}

// 获取应用数据目录
async function getAppDataDir(): Promise<string> {
	// 始终使用系统应用数据目录，让 Tauri 处理开发/生产环境差异
	return await appDataDir();
}

export function DataManagement() {
	const { state, actions } = useRouter();

	// 判断是否从系统页面进入
	const isFromSystemPage = state.source === "system";

	const [isExporting, setIsExporting] = useState(false);
	const [isImporting, setIsImporting] = useState(false);
	const [exportFormat, setExportFormat] = useState<string>("json");
	const [exportOptions, setExportOptions] = useState<ExportOptions>({
		include_categories: true,
		include_statistics: true,
		include_metadata: true,
		group_by_date: false,
		group_by_category: false,
	});
	const [dateRange, setDateRange] = useState({
		start: "",
		end: "",
	});
	const [lastExportResult, setLastExportResult] = useState<string>("");
	const [lastImportResult, setLastImportResult] = useState<string>("");
	const [backupPath, setBackupPath] = useState<string>("");

	// 备份设置本地状态
	const [backupSettings, setBackupSettings] = useState({
		autoBackup: true,
		backupInterval: 7,
		backupRetention: 30,
		backupDirectory: "",
	});

	const [statistics, setStatistics] = useState<{
		total_tasks: number;
		total_time_spent: number;
		total_transactions: number;
		total_notes: number;
		database_size: string;
		last_backup: string;
	}>({
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

	// 初始化时加载配置中的备份设置，并确保目录为绝对路径
	useEffect(() => {
		(async () => {
			try {
				const cfg: any = await invoke("get_config");

				// 读取备份目录，如果是相对路径则转换为绝对路径（相对应用数据目录）
				let dir = cfg?.data?.backup_directory ?? "";
				const isAbsolute =
					typeof dir === "string" &&
					(dir.startsWith("/") || /^[a-zA-Z]:[\\/]/.test(dir));

				try {
					if (!isAbsolute && dir) {
						const base = await getAppDataDir();
						dir = await join(base, dir);
					}

					if (!dir) {
						const base = await getAppDataDir();
						dir = await join(base, "backups");
					}
				} catch (err) {
					console.warn("无法获取默认备份目录", err);
				}

				setBackupSettings({
					autoBackup: cfg?.data?.auto_backup ?? true,
					backupInterval: cfg?.data?.backup_interval ?? 7,
					backupRetention: cfg?.data?.backup_retention ?? 30,
					backupDirectory: dir,
				});
			} catch (e) {
				console.error("读取配置失败", e);
			}
		})();
	}, []);

	// 获取当前（或默认）备份目录
	const getEffectiveBackupDir = useCallback(async () => {
		let dir = backupSettings.backupDirectory;

		if (!dir) {
			const base = await getAppDataDir();
			dir = await join(base, "backups");
		}

		// 转换为绝对路径，确保文件对话框能够正确识别
		dir = await pathResolve(dir);

		return dir;
	}, [backupSettings.backupDirectory]);

	// 获取当前（或默认）导出目录
	const getEffectiveExportDir = useCallback(async () => {
		const base = await getAppDataDir();
		const dir = await join(base, "exports");

		// 转换为绝对路径，确保文件对话框能够正确识别
		return await pathResolve(dir);
	}, []);

	// 选择备份目录
	const chooseBackupDirectory = useCallback(async () => {
		try {
			const defaultDir = await getEffectiveBackupDir();
			const dir = await open({ directory: true, defaultPath: defaultDir });
			if (dir && !Array.isArray(dir)) {
				setBackupSettings((prev) => ({ ...prev, backupDirectory: dir }));
			}
		} catch (e) {
			console.error(e);
		}
	}, [getEffectiveBackupDir]);

	// 保存备份设置到配置
	const handleSaveBackupSettings = useCallback(async () => {
		try {
			const cfg: any = await invoke("get_config");

			// 保存前再次确保备份目录为绝对路径
			let dir = backupSettings.backupDirectory;
			const isAbsolute =
				typeof dir === "string" &&
				(dir.startsWith("/") || /^[a-zA-Z]:[\\/]/.test(dir));

			if (!isAbsolute) {
				const base = await getAppDataDir();
				dir = await join(base, dir);
			}

			const updated = {
				...cfg,
				data: {
					...cfg.data,
					auto_backup: backupSettings.autoBackup,
					backup_interval: backupSettings.backupInterval,
					backup_retention: backupSettings.backupRetention,
					backup_directory: dir,
				},
			};

			await invoke("update_config", { config: updated });
			alert("备份设置已保存！");
		} catch (e) {
			console.error(e);
			alert("保存失败，请重试");
		}
	}, [backupSettings]);

	const handleExport = useCallback(async () => {
		try {
			setIsExporting(true);
			setLastExportResult("");

			// 获取默认导出目录
			const defaultDir = await getEffectiveExportDir();
			const filename = `lifetracker-export-${new Date().toISOString().split("T")[0]}.${exportFormat}`;
			const defaultPath = await join(defaultDir, filename);

			// 选择保存文件路径
			const filePath = await save({
				filters: [
					{
						name: "导出文件",
						extensions: [exportFormat],
					},
				],
				defaultPath,
			});

			if (!filePath) {
				return; // 用户取消了文件选择
			}

			// 准备导出选项
			const options: ExportOptions = {
				...exportOptions,
			};

			// 添加日期范围（如果设置了）
			if (dateRange.start && dateRange.end) {
				options.start_date = new Date(dateRange.start).toISOString();
				options.end_date = new Date(dateRange.end).toISOString();
			}

			// 调用后端导出命令
			const result = await invoke<string>("export_data", {
				format: exportFormat,
				filePath,
				options,
			});

			setLastExportResult(result);
		} catch (error) {
			console.error("导出失败:", error);
			setLastExportResult(`导出失败: ${error}`);
		} finally {
			setIsExporting(false);
		}
	}, [exportFormat, exportOptions, dateRange, getEffectiveExportDir]);

	const handleImport = useCallback(async () => {
		try {
			setIsImporting(true);
			setLastImportResult("");

			// 获取默认导出目录作为导入的默认位置
			const defaultDir = await getEffectiveExportDir();

			// 选择导入文件
			const filePath = await open({
				filters: [
					{
						name: "数据文件",
						extensions: ["json", "csv", "xml"],
					},
				],
				multiple: false,
				defaultPath: defaultDir,
			});

			if (!filePath) {
				return; // 用户取消了文件选择
			}

			if (!confirm("导入数据将覆盖现有数据，确定要继续吗？")) {
				return;
			}

			// 调用后端导入命令
			const result = await invoke<string>("import_data", {
				filePath,
			});

			setLastImportResult(result);
		} catch (error) {
			console.error("导入失败:", error);
			setLastImportResult(`导入失败: ${error}`);
		} finally {
			setIsImporting(false);
		}
	}, [getEffectiveExportDir]);

	const handleClearData = useCallback(async () => {
		if (
			!confirm(
				"这将删除所有数据，包括任务、分类和计时记录。此操作不可恢复，确定要继续吗？",
			)
		) {
			return;
		}

		try {
			await invoke("clear_all_data");
			alert("数据已清除！");
		} catch (error) {
			console.error("清除数据失败:", error);
			alert("清除失败，请重试。");
		}
	}, []);

	const handleOptionChange = useCallback(
		(key: keyof ExportOptions, value: boolean) => {
			setExportOptions((prev) => ({
				...prev,
				[key]: value,
			}));
		},
		[],
	);

	const handleBackup = useCallback(async () => {
		try {
			if (!backupSettings.backupDirectory) {
				alert("请先在下方选择备份目录，再执行立即备份。");
				return;
			}
			const now = new Date();
			const timestamp = now
				.toISOString()
				.replace(/T/, "_")
				.replace(/:/g, "-")
				.split(".")[0]; // YYYY-MM-DD_HH-MM-SS
			const path = await join(
				backupSettings.backupDirectory,
				`lifetracker-backup-${timestamp}.db`,
			);
			setBackupPath(path);
			const res = await invoke<string>("backup_database", {
				destPath: path,
			});
			alert(res);
		} catch (e) {
			console.error(e);
			alert("备份失败");
		}
	}, [backupSettings.backupDirectory]);

	const handleRestore = useCallback(async () => {
		try {
			const defaultDir = await getEffectiveBackupDir();
			const filePath = await open({
				filters: [
					{ name: "SQLite Backup", extensions: ["db", "sqlite", "bak"] },
				],
				multiple: false,
				defaultPath: defaultDir,
			});
			if (!filePath || Array.isArray(filePath)) return;
			if (!confirm("导入备份将覆盖当前数据库，确定继续？")) return;
			const res = await invoke<string>("restore_database", {
				srcPath: filePath,
			});
			alert(res + "\n请重启应用以生效");
		} catch (e) {
			console.error(e);
			alert("恢复失败");
		}
	}, [getEffectiveBackupDir]);

	// 获取数据统计信息
	const fetchStatistics = async () => {
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
	};

	useEffect(() => {
		fetchStatistics();
	}, []);

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

	return (
		<div className="h-full flex flex-col">
			{/* 固定顶部导航栏 */}
			<div className="flex-shrink-0 px-4 md:px-6 py-4 border-b border-gray-200 dark:border-gray-700 surface-adaptive">
				<div className="flex items-center justify-between">
					<div className="flex items-center space-x-3">
						{/* 仅在从系统页面进入时显示返回按钮 */}
						{isFromSystemPage && (
							<button
								onClick={actions.goBack}
								className="flex items-center justify-center w-8 h-8 text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
								title="返回"
							>
								<ArrowLeft className="w-5 h-5" />
							</button>
						)}
						<h1 className="text-2xl font-bold text-gray-900 dark:text-white">
							数据
						</h1>
					</div>
				</div>
			</div>

			{/* 可滚动内容区域 */}
			<div className="flex-1 overflow-y-auto py-4 px-4 md:px-6 scroll-container">
				<div className="space-y-6">
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

					{loading && (
						<div className="flex items-center justify-center py-8">
							<div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
							<span className="ml-3 text-gray-600 dark:text-gray-400">
								处理中...
							</span>
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
								<div className="text-lg font-bold text-gray-900 dark:text-gray-100">
									{statistics.last_backup}
								</div>
							</div>
						</div>
					</div>

					<div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
						{/* 数据导出 */}
						<details
							className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6"
							open={false}
						>
							<summary className="flex items-center outline-none cursor-pointer select-none">
								<Download className="h-5 w-5 text-green-600 dark:text-green-400 mr-2 flex-shrink-0" />
								<span className="text-xl font-semibold text-gray-900 dark:text-gray-100">
									数据导出
								</span>
							</summary>
							<div className="mt-4 space-y-6">
								{/* 导出格式选择 */}
								<div className="mb-6">
									<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
										<FileText className="h-4 w-4 inline mr-1" />
										导出格式
									</label>
									<select
										value={exportFormat}
										onChange={(e) => {
											const newFormat = e.target.value;
											setExportFormat(newFormat);
										}}
										className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
									>
										<option value="json">JSON - 结构化数据</option>
										<option value="csv">CSV - 表格数据</option>
										<option value="xml">XML - 标记语言</option>
										<option value="html">HTML - 网页格式</option>
										<option value="markdown">Markdown - 文档格式</option>
									</select>
								</div>

								{/* 日期范围选择 */}
								<div className="mb-6">
									<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
										<Calendar className="h-4 w-4 inline mr-1" />
										日期范围（可选）
									</label>
									<div className="grid grid-cols-2 gap-4">
										<div>
											<label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
												开始日期
											</label>
											<input
												type="date"
												value={dateRange.start}
												onChange={(e) =>
													setDateRange((prev) => ({
														...prev,
														start: e.target.value,
													}))
												}
												className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
											/>
										</div>
										<div>
											<label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
												结束日期
											</label>
											<input
												type="date"
												value={dateRange.end}
												onChange={(e) =>
													setDateRange((prev) => ({
														...prev,
														end: e.target.value,
													}))
												}
												className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
											/>
										</div>
									</div>
								</div>

								{/* 导出选项 */}
								<div className="mb-6">
									<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
										<Settings className="h-4 w-4 inline mr-1" />
										导出选项
									</label>
									<div className="space-y-3">
										<label className="flex items-center">
											<input
												type="checkbox"
												checked={exportOptions.include_categories ?? true}
												onChange={(e) =>
													handleOptionChange(
														"include_categories",
														e.target.checked,
													)
												}
												className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
											/>
											<span className="ml-2 text-sm text-gray-700 dark:text-gray-300">
												包含分类信息
											</span>
										</label>

										<label className="flex items-center">
											<input
												type="checkbox"
												checked={exportOptions.include_statistics ?? true}
												onChange={(e) =>
													handleOptionChange(
														"include_statistics",
														e.target.checked,
													)
												}
												className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
											/>
											<span className="ml-2 text-sm text-gray-700 dark:text-gray-300">
												包含统计数据
											</span>
										</label>

										<label className="flex items-center">
											<input
												type="checkbox"
												checked={exportOptions.include_metadata ?? true}
												onChange={(e) =>
													handleOptionChange(
														"include_metadata",
														e.target.checked,
													)
												}
												className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
											/>
											<span className="ml-2 text-sm text-gray-700 dark:text-gray-300">
												包含元数据
											</span>
										</label>

										<label className="flex items-center">
											<input
												type="checkbox"
												checked={exportOptions.group_by_date ?? false}
												onChange={(e) =>
													handleOptionChange("group_by_date", e.target.checked)
												}
												className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
											/>
											<span className="ml-2 text-sm text-gray-700 dark:text-gray-300">
												按日期分组
											</span>
										</label>

										<label className="flex items-center">
											<input
												type="checkbox"
												checked={exportOptions.group_by_category ?? false}
												onChange={(e) =>
													handleOptionChange(
														"group_by_category",
														e.target.checked,
													)
												}
												className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
											/>
											<span className="ml-2 text-sm text-gray-700 dark:text-gray-300">
												按分类分组
											</span>
										</label>
									</div>
								</div>

								{/* 导出按钮 */}
								<button
									onClick={handleExport}
									disabled={isExporting}
									className={`w-full px-4 py-2 rounded-md font-medium text-white transition-colors ${
										isExporting
											? "bg-gray-400 cursor-not-allowed"
											: "bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2"
									}`}
								>
									{isExporting ? (
										<span className="flex items-center justify-center">
											<svg
												className="animate-spin -ml-1 mr-3 h-5 w-5 text-white"
												xmlns="http://www.w3.org/2000/svg"
												fill="none"
												viewBox="0 0 24 24"
											>
												<circle
													className="opacity-25"
													cx="12"
													cy="12"
													r="10"
													stroke="currentColor"
													strokeWidth="4"
												/>
												<path
													className="opacity-75"
													fill="currentColor"
													d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
												/>
											</svg>
											导出中...
										</span>
									) : (
										<span className="flex items-center justify-center">
											<Download className="h-4 w-4 mr-2" />
											开始导出
										</span>
									)}
								</button>

								{/* 导出结果 */}
								{lastExportResult && (
									<div
										className={`mt-4 p-3 rounded-md ${
											lastExportResult.includes("失败")
												? "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800"
												: "bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800"
										}`}
									>
										<div className="flex items-start">
											{lastExportResult.includes("失败") ? (
												<AlertCircle className="h-4 w-4 text-red-600 dark:text-red-400 mr-2 mt-0.5 flex-shrink-0" />
											) : (
												<CheckCircle className="h-4 w-4 text-green-600 dark:text-green-400 mr-2 mt-0.5 flex-shrink-0" />
											)}
											<p
												className={`text-sm ${
													lastExportResult.includes("失败")
														? "text-red-700 dark:text-red-300"
														: "text-green-700 dark:text-green-300"
												}`}
											>
												{lastExportResult}
											</p>
										</div>
									</div>
								)}
							</div>
						</details>

						{/* 数据导入 */}
						<details
							className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6"
							open={false}
						>
							<summary className="flex items-center outline-none cursor-pointer select-none">
								<Upload className="h-5 w-5 text-blue-600 dark:text-blue-400 mr-2 flex-shrink-0" />
								<span className="text-xl font-semibold text-gray-900 dark:text-gray-100">
									数据导入
								</span>
							</summary>
							<div className="mt-4 space-y-6">
								<div className="mt-2">
									<div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-md p-4">
										<div className="flex items-start">
											<Info className="h-4 w-4 text-yellow-600 dark:text-yellow-400 mr-2 mt-0.5 flex-shrink-0" />
											<div className="text-sm text-yellow-700 dark:text-yellow-300">
												<p className="font-medium mb-1">导入注意事项：</p>
												<ul className="list-disc list-inside space-y-1">
													<li>导入操作将覆盖现有数据</li>
													<li>支持 JSON、CSV、XML 格式</li>
													<li>建议在导入前先导出备份</li>
													<li>大文件导入可能需要较长时间</li>
												</ul>
											</div>
										</div>
									</div>
								</div>

								<button
									onClick={handleImport}
									disabled={isImporting}
									className={`w-full px-4 py-2 rounded-md font-medium text-white transition-colors ${
										isImporting
											? "bg-gray-400 cursor-not-allowed"
											: "bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
									}`}
								>
									{isImporting ? (
										<span className="flex items-center justify-center">
											<svg
												className="animate-spin -ml-1 mr-3 h-5 w-5 text-white"
												xmlns="http://www.w3.org/2000/svg"
												fill="none"
												viewBox="0 0 24 24"
											>
												<circle
													className="opacity-25"
													cx="12"
													cy="12"
													r="10"
													stroke="currentColor"
													strokeWidth="4"
												/>
												<path
													className="opacity-75"
													fill="currentColor"
													d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
												/>
											</svg>
											导入中...
										</span>
									) : (
										<span className="flex items-center justify-center">
											<Upload className="h-4 w-4 mr-2" />
											选择文件导入
										</span>
									)}
								</button>

								{/* 导入结果 */}
								{lastImportResult && (
									<div
										className={`mt-4 p-3 rounded-md ${
											lastImportResult.includes("失败")
												? "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800"
												: "bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800"
										}`}
									>
										<div className="flex items-start">
											{lastImportResult.includes("失败") ? (
												<AlertCircle className="h-4 w-4 text-red-600 dark:text-red-400 mr-2 mt-0.5 flex-shrink-0" />
											) : (
												<CheckCircle className="h-4 w-4 text-green-600 dark:text-green-400 mr-2 mt-0.5 flex-shrink-0" />
											)}
											<p
												className={`text-sm ${
													lastImportResult.includes("失败")
														? "text-red-700 dark:text-red-300"
														: "text-green-700 dark:text-green-300"
												}`}
											>
												{lastImportResult}
											</p>
										</div>
									</div>
								)}
							</div>
						</details>
					</div>

					{/* 数据清理 */}
					<details
						className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6 mt-6"
						open={false}
					>
						<summary className="flex items-center outline-none cursor-pointer select-none">
							<RefreshCw className="h-5 w-5 text-red-600 dark:text-red-400 mr-2 flex-shrink-0" />
							<span className="text-xl font-semibold text-gray-900 dark:text-gray-100">
								数据清理
							</span>
						</summary>
						<div className="mt-4 space-y-6">
							<div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-4 mt-2">
								<div className="flex items-start">
									<AlertCircle className="h-4 w-4 text-red-600 dark:text-red-400 mr-2 mt-0.5 flex-shrink-0" />
									<div className="text-sm text-red-700 dark:text-red-300">
										<p className="font-medium mb-1">危险操作警告：</p>
										<p>
											此操作将永久删除所有数据，包括任务记录、分类信息和统计数据。操作不可恢复，请谨慎使用。
										</p>
									</div>
								</div>
							</div>

							<button
								onClick={handleClearData}
								className="px-6 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2 transition-colors"
							>
								<RefreshCw className="h-4 w-4 inline mr-2" />
								清除所有数据
							</button>
						</div>
					</details>

					{/* 备份与恢复 */}
					<details
						className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6 mt-6"
						open={false}
					>
						<summary className="flex items-center outline-none cursor-pointer select-none">
							<RefreshCw className="h-5 w-5 text-orange-600 dark:text-orange-400 mr-2 flex-shrink-0" />
							<span className="text-xl font-semibold text-gray-900 dark:text-gray-100">
								备份与恢复
							</span>
						</summary>

						<div className="mt-4 space-y-6">
							{/* 自动备份设置 */}
							<label className="flex items-center">
								<input
									type="checkbox"
									checked={backupSettings.autoBackup}
									onChange={(e) =>
										setBackupSettings((prev) => ({
											...prev,
											autoBackup: e.target.checked,
										}))
									}
									className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
								/>
								<span className="ml-2 text-sm text-gray-700 dark:text-gray-300">
									启用自动备份
								</span>
							</label>

							<div className="grid grid-cols-2 gap-4">
								<div>
									<label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
										备份间隔（天）
									</label>
									<input
										type="number"
										min={1}
										value={backupSettings.backupInterval}
										onChange={(e) =>
											setBackupSettings((prev) => ({
												...prev,
												backupInterval: Number(e.target.value),
											}))
										}
										className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
									/>
								</div>
								<div>
									<label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
										保留备份数量
									</label>
									<input
										type="number"
										min={1}
										value={backupSettings.backupRetention}
										onChange={(e) =>
											setBackupSettings((prev) => ({
												...prev,
												backupRetention: Number(e.target.value),
											}))
										}
										className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
									/>
								</div>
							</div>

							{/* 备份目录选择 */}
							<div>
								<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
									备份目录
								</label>
								<div className="flex items-center space-x-2">
									<input
										type="text"
										readOnly
										value={backupSettings.backupDirectory}
										className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-gray-50 dark:bg-gray-800 text-gray-500 dark:text-gray-400"
									/>
									<button
										onClick={chooseBackupDirectory}
										className="px-3 py-2 bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 rounded-md text-sm"
									>
										选择
									</button>
								</div>
							</div>

							{/* 保存设置 */}
							<button
								onClick={handleSaveBackupSettings}
								className="w-full px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 transition-colors"
							>
								保存备份设置
							</button>

							<hr className="border-gray-200 dark:border-gray-700" />

							{/* 手动备份与恢复 */}
							<div className="space-y-4">
								<button
									onClick={handleBackup}
									className="w-full px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-colors"
								>
									立即备份
								</button>

								<button
									onClick={handleRestore}
									className="w-full px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2 transition-colors"
								>
									从备份恢复
								</button>

								{backupPath && (
									<p className="text-xs text-gray-500 dark:text-gray-400 text-center">
										最近备份文件: {backupPath}
									</p>
								)}
							</div>
						</div>
					</details>

					{/* 多端同步 */}
					<details
						className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6 mt-6"
						open={false}
					>
						<summary className="flex items-center outline-none cursor-pointer select-none">
							<Cloud className="h-5 w-5 text-indigo-600 dark:text-indigo-400 mr-2 flex-shrink-0" />
							<span className="text-xl font-semibold text-gray-900 dark:text-gray-100">
								多端同步
							</span>
						</summary>
						<SyncManagement />
					</details>
				</div>
			</div>
		</div>
	);
}

// 同步管理组件
function SyncManagement() {
	const [syncConfig, setSyncConfig] = useState({
		enabled: false,
		provider: "webdav",
		auto_sync: false,
		sync_interval: 30,
		conflict_strategy: "manual",
		webdav_config: {
			url: "",
			username: "",
			password: "",
			directory: "LifeTracker",
		},
	});

	const [syncStatus, setSyncStatus] = useState({
		status: "disabled",
		is_syncing: false,
		last_sync_time: null,
		next_sync_time: null,
		error_message: null,
	});

	const [isLoading, setIsLoading] = useState(false);
	const [isTesting, setIsTesting] = useState(false);
	const [testResult, setTestResult] = useState("");
	const [showPassword, setShowPassword] = useState(false);

	// 获取同步配置
	const fetchSyncConfig = useCallback(async () => {
		try {
			const config = (await invoke("get_sync_config")) as any;
			setSyncConfig(config);
		} catch (error) {
			console.error("获取同步配置失败:", error);
		}
	}, []);

	// 获取同步状态
	const fetchSyncStatus = useCallback(async () => {
		try {
			const status = (await invoke("get_sync_status")) as any;
			setSyncStatus(status);
		} catch (error) {
			console.error("获取同步状态失败:", error);
		}
	}, []);

	// 初始化
	useEffect(() => {
		fetchSyncConfig();
		fetchSyncStatus();
	}, [fetchSyncConfig, fetchSyncStatus]);

	// 保存配置
	const handleSaveConfig = useCallback(async () => {
		setIsLoading(true);
		try {
			await invoke("save_sync_config", { request: syncConfig });
			alert("同步配置已保存！");
			// 重新获取配置和状态，确保UI状态同步
			await Promise.all([fetchSyncConfig(), fetchSyncStatus()]);
		} catch (error) {
			console.error("保存同步配置失败:", error);
			alert("保存失败，请重试");
		} finally {
			setIsLoading(false);
		}
	}, [syncConfig, fetchSyncConfig, fetchSyncStatus]);

	// 测试连接
	const handleTestConnection = useCallback(async () => {
		setIsTesting(true);
		setTestResult("");
		try {
			const result = await invoke("test_sync_connection", {
				request: syncConfig,
			});
			setTestResult(`✅ ${result}`);
		} catch (error) {
			setTestResult(`❌ ${error}`);
		} finally {
			setIsTesting(false);
		}
	}, [syncConfig]);

	// 开始同步
	const handleStartSync = useCallback(async () => {
		if (!syncConfig.enabled) {
			// 不执行任何操作，按钮文案已经显示了提示
			return;
		}

		setIsLoading(true);
		try {
			const result = await invoke("start_sync");
			alert(`同步成功：${result}`);
			await fetchSyncStatus();
		} catch (error) {
			console.error("同步失败:", error);
			alert(`同步失败：${error}`);
		} finally {
			setIsLoading(false);
		}
	}, [syncConfig.enabled, fetchSyncStatus]);

	return (
		<div className="mt-4 space-y-6">
			{/* 同步状态 */}
			<div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
				<h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-3">
					同步状态
				</h3>
				<div className="grid grid-cols-2 gap-4">
					<div>
						<div className="text-sm text-gray-600 dark:text-gray-400">
							当前状态
						</div>
						<div
							className={`text-sm font-medium ${
								syncStatus.status === "enabled"
									? "text-green-600 dark:text-green-400"
									: "text-gray-600 dark:text-gray-400"
							}`}
						>
							{syncStatus.status === "enabled" ? "已启用" : "已禁用"}
						</div>
					</div>
					<div>
						<div className="text-sm text-gray-600 dark:text-gray-400">
							最后同步
						</div>
						<div className="text-sm font-medium text-gray-900 dark:text-gray-100">
							{syncStatus.last_sync_time || "从未"}
						</div>
					</div>
				</div>

				{syncStatus.error_message && (
					<div className="mt-3 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
						<div className="text-sm text-red-700 dark:text-red-300">
							{syncStatus.error_message}
						</div>
					</div>
				)}
			</div>

			{/* 基本配置 */}
			<div className="space-y-4">
				<h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">
					基本配置
				</h3>

				<label className="flex items-center">
					<input
						type="checkbox"
						checked={syncConfig.enabled}
						onChange={(e) =>
							setSyncConfig((prev) => ({
								...prev,
								enabled: e.target.checked,
							}))
						}
						className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
					/>
					<span className="ml-2 text-sm text-gray-700 dark:text-gray-300">
						启用多端同步
					</span>
				</label>

				<div>
					<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
						同步提供者
					</label>
					<select
						value={syncConfig.provider}
						onChange={(e) =>
							setSyncConfig((prev) => ({
								...prev,
								provider: e.target.value,
							}))
						}
						className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
					>
						<option value="webdav">WebDAV</option>
						<option value="github" disabled>
							GitHub (开发中)
						</option>
						<option value="local" disabled>
							本地网络 (开发中)
						</option>
					</select>
				</div>

				<label className="flex items-center">
					<input
						type="checkbox"
						checked={syncConfig.auto_sync}
						onChange={(e) =>
							setSyncConfig((prev) => ({
								...prev,
								auto_sync: e.target.checked,
							}))
						}
						className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
					/>
					<span className="ml-2 text-sm text-gray-700 dark:text-gray-300">
						启用自动同步
					</span>
				</label>

				<div>
					<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
						同步间隔（分钟）
					</label>
					<input
						type="number"
						min={5}
						max={1440}
						value={syncConfig.sync_interval}
						onChange={(e) =>
							setSyncConfig((prev) => ({
								...prev,
								sync_interval: Number(e.target.value),
							}))
						}
						className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
					/>
				</div>

				<div>
					<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
						冲突解决策略
					</label>
					<select
						value={syncConfig.conflict_strategy}
						onChange={(e) =>
							setSyncConfig((prev) => ({
								...prev,
								conflict_strategy: e.target.value,
							}))
						}
						className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
					>
						<option value="manual">手动解决</option>
						<option value="local_wins">本地优先</option>
						<option value="remote_wins">远程优先</option>
						<option value="keep_both">保留两个版本</option>
					</select>
				</div>
			</div>

			{/* WebDAV 配置 - 只在启用同步时显示 */}
			{syncConfig.enabled && syncConfig.provider === "webdav" && (
				<div className="space-y-4">
					<h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">
						WebDAV 配置
					</h3>

					<div>
						<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
							服务器 URL
						</label>
						<input
							type="url"
							placeholder="https://example.com/webdav"
							value={syncConfig.webdav_config.url}
							onChange={(e) =>
								setSyncConfig((prev) => ({
									...prev,
									webdav_config: {
										...prev.webdav_config,
										url: e.target.value,
									},
								}))
							}
							className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
						/>
					</div>

					<div>
						<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
							用户名
						</label>
						<input
							type="text"
							value={syncConfig.webdav_config.username}
							onChange={(e) =>
								setSyncConfig((prev) => ({
									...prev,
									webdav_config: {
										...prev.webdav_config,
										username: e.target.value,
									},
								}))
							}
							className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
						/>
					</div>

					<div>
						<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
							密码
						</label>
						<div className="relative">
							<input
								type={showPassword ? "text" : "password"}
								value={syncConfig.webdav_config.password}
								onChange={(e) =>
									setSyncConfig((prev) => ({
										...prev,
										webdav_config: {
											...prev.webdav_config,
											password: e.target.value,
										},
									}))
								}
								className="w-full px-3 py-2 pr-10 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
							/>
							<button
								type="button"
								onClick={() => setShowPassword(!showPassword)}
								className="absolute inset-y-0 right-0 flex items-center pr-3"
							>
								{showPassword ? (
									<span className="text-gray-400">🙈</span>
								) : (
									<span className="text-gray-400">👁️</span>
								)}
							</button>
						</div>
					</div>

					<div>
						<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
							同步目录
						</label>
						<input
							type="text"
							value={syncConfig.webdav_config.directory}
							onChange={(e) =>
								setSyncConfig((prev) => ({
									...prev,
									webdav_config: {
										...prev.webdav_config,
										directory: e.target.value,
									},
								}))
							}
							className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
						/>
					</div>

					{/* 测试连接 */}
					<div className="space-y-2">
						<button
							onClick={handleTestConnection}
							disabled={isTesting}
							className={`px-4 py-2 rounded-md font-medium text-white transition-colors ${
								isTesting
									? "bg-gray-400 cursor-not-allowed"
									: "bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
							}`}
						>
							{isTesting ? "测试中..." : "测试连接"}
						</button>

						{testResult && (
							<div
								className={`p-3 rounded-md text-sm ${
									testResult.startsWith("✅")
										? "bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 text-green-700 dark:text-green-300"
										: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-300"
								}`}
							>
								{testResult}
							</div>
						)}
					</div>
				</div>
			)}

			{/* 同步未启用时的提示 */}
			{!syncConfig.enabled && (
				<div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
					<div className="flex items-center">
						<Info className="h-5 w-5 text-gray-400 mr-2" />
						<span className="text-sm text-gray-600 dark:text-gray-400">
							请先启用多端同步功能以配置同步设置
						</span>
					</div>
				</div>
			)}

			{/* 操作按钮 */}
			<div className="flex space-x-4">
				<button
					onClick={handleSaveConfig}
					disabled={isLoading}
					className={`px-6 py-2 rounded-md font-medium text-white transition-colors ${
						isLoading
							? "bg-gray-400 cursor-not-allowed"
							: "bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500"
					}`}
				>
					{isLoading ? "保存中..." : "保存配置"}
				</button>

				<button
					onClick={handleStartSync}
					disabled={isLoading || !syncConfig.enabled}
					className={`px-6 py-2 rounded-md font-medium text-white transition-colors ${
						isLoading || !syncConfig.enabled
							? "bg-gray-400 cursor-not-allowed"
							: "bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500"
					}`}
				>
					{syncStatus.is_syncing
						? "同步中..."
						: !syncConfig.enabled
							? "请先启用同步"
							: "立即同步"}
				</button>
			</div>

			{/* 提示信息 */}
			<div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-md p-4">
				<div className="flex items-start">
					<Info className="h-4 w-4 text-blue-600 dark:text-blue-400 mr-2 mt-0.5 flex-shrink-0" />
					<div className="text-sm text-blue-700 dark:text-blue-300">
						<p className="font-medium mb-1">同步功能说明：</p>
						<ul className="list-disc list-inside space-y-1">
							<li>支持 WebDAV 协议的云存储服务（如 Nextcloud、ownCloud）</li>
							<li>密码会进行加密存储，确保安全性</li>
							<li>冲突解决：手动处理可让您选择保留哪个版本的数据</li>
							<li>建议首次同步前先备份本地数据</li>
						</ul>
					</div>
				</div>
			</div>
		</div>
	);
}
