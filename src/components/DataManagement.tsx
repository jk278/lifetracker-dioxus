import { invoke } from "@tauri-apps/api/core";
import { appDataDir, join, resolve as pathResolve } from "@tauri-apps/api/path";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
	AlertCircle,
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
import { useScrollbarHiding } from "../hooks/useScrollbarHiding";

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
	// 滚动条隐藏hook
	const scrollRef = useScrollbarHiding<HTMLDivElement>();
	
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

	return (
		<div
			ref={scrollRef}
			className="h-full overflow-y-auto py-4 px-4 md:px-6 scroll-container"
		>
			<div className="space-y-6">
				{/* 页面标题 */}
				<h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
					数据管理
				</h1>

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
												handleOptionChange("include_metadata", e.target.checked)
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

				{/* 同步（占位） */}
				<details
					className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6 mt-6"
					open={false}
				>
					<summary className="flex items-center outline-none cursor-pointer select-none">
						<Cloud className="h-5 w-5 text-indigo-600 dark:text-indigo-400 mr-2 flex-shrink-0" />
						<span className="text-xl font-semibold text-gray-900 dark:text-gray-100">
							多端同步 (开发中)
						</span>
					</summary>
					<div className="mt-4 text-sm text-gray-600 dark:text-gray-400">
						<p className="mb-4">
							未来版本将支持将数据同步至云端（GitHub / WebDAV / Supabase
							等）。敬请期待！
						</p>
						<button
							disabled
							className="px-4 py-2 bg-indigo-400/60 text-white rounded-md cursor-not-allowed"
						>
							功能开发中
						</button>
					</div>
				</details>
			</div>
		</div>
	);
}
