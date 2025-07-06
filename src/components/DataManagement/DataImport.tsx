import { invoke } from "@tauri-apps/api/core";
import { appDataDir, join, resolve as pathResolve } from "@tauri-apps/api/path";
import { open } from "@tauri-apps/plugin-dialog";
import {
	AlertCircle,
	ArrowLeft,
	CheckCircle,
	Info,
	Upload,
} from "lucide-react";
import { useCallback, useState } from "react";
import { useNavigation } from "../../hooks/useRouter";

// 获取应用数据目录
async function getAppDataDir(): Promise<string> {
	return await appDataDir();
}

export function DataImport() {
	const { canGoBack, goBack } = useNavigation();

	const [isImporting, setIsImporting] = useState(false);
	const [lastImportResult, setLastImportResult] = useState<string>("");

	// 获取默认导出目录
	const getEffectiveExportDir = useCallback(async () => {
		const base = await getAppDataDir();
		const dir = await join(base, "exports");
		return await pathResolve(dir);
	}, []);

	// 处理导入
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
							数据导入
						</h1>
					</div>
				</div>
			</div>

			{/* 可滚动内容区域 */}
			<div className="flex-1 overflow-y-auto py-4 px-4 md:px-6 scroll-container">
				<div className="max-w-2xl mx-auto space-y-6">
					{/* 导入注意事项 */}
					<div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-6">
						<div className="flex items-start">
							<Info className="h-5 w-5 text-yellow-600 dark:text-yellow-400 mr-3 mt-0.5 flex-shrink-0" />
							<div className="text-sm text-yellow-700 dark:text-yellow-300">
								<p className="font-medium mb-2">导入注意事项：</p>
								<ul className="list-disc list-inside space-y-1">
									<li>导入操作将覆盖现有数据</li>
									<li>支持 JSON、CSV、XML 格式</li>
									<li>建议在导入前先导出备份</li>
									<li>大文件导入可能需要较长时间</li>
								</ul>
							</div>
						</div>
					</div>

					{/* 导入操作 */}
					<div className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6">
						<div className="text-center space-y-4">
							<div className="flex justify-center">
								<div className="w-16 h-16 bg-blue-100 dark:bg-blue-900/30 rounded-full flex items-center justify-center">
									<Upload className="w-8 h-8 text-blue-600 dark:text-blue-400" />
								</div>
							</div>

							<div>
								<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
									选择数据文件
								</h3>
								<p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
									支持 JSON、CSV、XML 格式的数据文件
								</p>
							</div>

							<button
								onClick={handleImport}
								disabled={isImporting}
								className={`w-full px-6 py-3 rounded-lg font-medium text-white transition-colors ${
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
										<Upload className="h-5 w-5 mr-2" />
										选择文件导入
									</span>
								)}
							</button>
						</div>
					</div>

					{/* 导入结果 */}
					{lastImportResult && (
						<div
							className={`surface-adaptive rounded-lg border p-4 ${
								lastImportResult.includes("失败")
									? "border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-900/20"
									: "border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-900/20"
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

					{/* 支持的文件格式说明 */}
					<div className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6">
						<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
							支持的文件格式
						</h3>
						<div className="grid grid-cols-1 md:grid-cols-3 gap-4">
							<div className="text-center p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
								<div className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-1">
									JSON
								</div>
								<div className="text-xs text-gray-600 dark:text-gray-400">
									结构化数据格式
								</div>
							</div>
							<div className="text-center p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
								<div className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-1">
									CSV
								</div>
								<div className="text-xs text-gray-600 dark:text-gray-400">
									逗号分隔表格
								</div>
							</div>
							<div className="text-center p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
								<div className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-1">
									XML
								</div>
								<div className="text-xs text-gray-600 dark:text-gray-400">
									标记语言格式
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
}
