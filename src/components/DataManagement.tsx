import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
	AlertCircle,
	Calendar,
	CheckCircle,
	Database,
	Download,
	FileText,
	Info,
	RefreshCw,
	Settings,
	Upload,
} from "lucide-react";
import React, { useCallback, useState } from "react";

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

export function DataManagement() {
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

	const handleExport = useCallback(async () => {
		try {
			setIsExporting(true);
			setLastExportResult("");

			// 选择保存文件路径
			const filePath = await save({
				filters: [
					{
						name: "导出文件",
						extensions: [exportFormat],
					},
				],
				defaultPath: `lifetracker-export-${new Date().toISOString().split("T")[0]}.${exportFormat}`,
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
	}, [exportFormat, exportOptions, dateRange]);

	const handleImport = useCallback(async () => {
		try {
			setIsImporting(true);
			setLastImportResult("");

			// 选择导入文件
			const filePath = await open({
				filters: [
					{
						name: "数据文件",
						extensions: ["json", "csv", "xml"],
					},
				],
				multiple: false,
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
	}, []);

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

	return (
		<div className="space-y-6">
			{/* 页面标题 */}
			<div className="flex items-center justify-between">
				<div className="flex items-center">
					<Database className="h-6 w-6 text-blue-600 dark:text-blue-400 mr-3" />
					<h1 className="text-2xl font-bold text-gray-900 dark:text-white">
						数据管理
					</h1>
				</div>
			</div>

			<div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
				{/* 数据导出 */}
				<div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
					<div className="flex items-center mb-6">
						<Download className="h-5 w-5 text-green-600 dark:text-green-400 mr-2" />
						<h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100">
							数据导出
						</h2>
					</div>

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
										setDateRange((prev) => ({ ...prev, start: e.target.value }))
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
										setDateRange((prev) => ({ ...prev, end: e.target.value }))
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

				{/* 数据导入 */}
				<div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
					<div className="flex items-center mb-6">
						<Upload className="h-5 w-5 text-blue-600 dark:text-blue-400 mr-2" />
						<h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100">
							数据导入
						</h2>
					</div>

					<div className="mb-6">
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
			</div>

			{/* 数据清理 */}
			<div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
				<div className="flex items-center mb-6">
					<RefreshCw className="h-5 w-5 text-red-600 dark:text-red-400 mr-2" />
					<h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100">
						数据清理
					</h2>
				</div>

				<div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-4 mb-6">
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

			{/* 格式说明 */}
			<div className="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-6">
				<h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-4">
					<Info className="h-5 w-5 inline mr-2" />
					导出格式说明
				</h3>
				<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 text-sm text-gray-600 dark:text-gray-400">
					<div className="bg-white dark:bg-gray-800 p-3 rounded border">
						<p className="font-medium text-gray-900 dark:text-gray-100">
							JSON 格式
						</p>
						<p>结构化数据，适合程序处理和数据交换</p>
					</div>
					<div className="bg-white dark:bg-gray-800 p-3 rounded border">
						<p className="font-medium text-gray-900 dark:text-gray-100">
							CSV 格式
						</p>
						<p>表格数据，可在 Excel 等软件中打开</p>
					</div>
					<div className="bg-white dark:bg-gray-800 p-3 rounded border">
						<p className="font-medium text-gray-900 dark:text-gray-100">
							HTML 格式
						</p>
						<p>网页格式，可在浏览器中查看，带样式</p>
					</div>
					<div className="bg-white dark:bg-gray-800 p-3 rounded border">
						<p className="font-medium text-gray-900 dark:text-gray-100">
							Markdown 格式
						</p>
						<p>文档格式，适合编辑和分享</p>
					</div>
					<div className="bg-white dark:bg-gray-800 p-3 rounded border">
						<p className="font-medium text-gray-900 dark:text-gray-100">
							XML 格式
						</p>
						<p>标准化数据交换格式</p>
					</div>
				</div>
			</div>
		</div>
	);
}
