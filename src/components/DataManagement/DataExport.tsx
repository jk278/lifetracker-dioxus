import { invoke } from "@tauri-apps/api/core";
import { appDataDir, join, resolve as pathResolve } from "@tauri-apps/api/path";
import { save } from "@tauri-apps/plugin-dialog";
import {
	AlertCircle,
	ArrowLeft,
	Calendar,
	CheckCircle,
	Download,
	FileText,
	Settings,
} from "lucide-react";
import { useCallback, useState } from "react";
import { useNavigation } from "../../hooks/useRouter";

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
	return await appDataDir();
}

export function DataExport() {
	const { canGoBack, goBack } = useNavigation();

	const [isExporting, setIsExporting] = useState(false);
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

	// 获取默认导出目录
	const getEffectiveExportDir = useCallback(async () => {
		const base = await getAppDataDir();
		const dir = await join(base, "exports");
		return await pathResolve(dir);
	}, []);

	// 处理导出选项变化
	const handleOptionChange = useCallback(
		(key: keyof ExportOptions, value: boolean) => {
			setExportOptions((prev) => ({
				...prev,
				[key]: value,
			}));
		},
		[],
	);

	// 处理导出
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

	// 返回处理
	const handleBack = useCallback(() => {
		if (canGoBack) {
			goBack();
		}
	}, [canGoBack, goBack]);

	return (
		<div className="h-full flex flex-col">
			{/* 固定顶部导航栏 */}
			<div className="flex-shrink-0 px-4 md:px-6 py-4 border-b border-gray-200 dark:border-gray-700 surface-adaptive">
				<div className="flex items-center justify-between">
					<div className="flex items-center space-x-3">
						<button
							onClick={handleBack}
							className="flex items-center justify-center w-8 h-8 text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
							title="返回"
						>
							<ArrowLeft className="w-5 h-5" />
						</button>
						<h1 className="text-2xl font-bold text-gray-900 dark:text-white">
							数据导出
						</h1>
					</div>
				</div>
			</div>

			{/* 可滚动内容区域 */}
			<div className="flex-1 overflow-y-auto py-4 px-4 md:px-6 scroll-container">
				<div className="max-w-2xl mx-auto space-y-6">
					{/* 导出格式选择 */}
					<div className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6">
						<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
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
					<div className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6">
						<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
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
					<div className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6">
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
										handleOptionChange("include_categories", e.target.checked)
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
										handleOptionChange("include_statistics", e.target.checked)
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
										handleOptionChange("group_by_category", e.target.checked)
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
					<div className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6">
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
					</div>

					{/* 导出结果 */}
					{lastExportResult && (
						<div
							className={`surface-adaptive rounded-lg border p-4 ${
								lastExportResult.includes("失败")
									? "border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-900/20"
									: "border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-900/20"
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
			</div>
		</div>
	);
}
