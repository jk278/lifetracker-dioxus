import { invoke } from "@tauri-apps/api/core";
import { AlertCircle, ArrowLeft, Database, Trash2 } from "lucide-react";
import { useCallback, useState } from "react";
import { useNavigation } from "../../hooks/useRouter";

export function DataCleanup() {
	const { canGoBack, goBack } = useNavigation();
	const [isClearing, setIsClearing] = useState(false);
	const [showConfirmDialog, setShowConfirmDialog] = useState(false);
	const [confirmInput, setConfirmInput] = useState("");

	// 数据清理操作
	const handleClearData = useCallback(async () => {
		setShowConfirmDialog(true);
	}, []);

	// 确认清除操作
	const handleConfirmClear = useCallback(async () => {
		if (confirmInput !== "DELETE") {
			return;
		}

		setIsClearing(true);
		setShowConfirmDialog(false);
		setConfirmInput("");

		try {
			await invoke("clear_all_data");
			alert("数据已清除！建议立即重启应用。");
			// 清除成功后导航回数据管理页面
			if (canGoBack) {
				goBack();
			}
		} catch (error) {
			console.error("清除数据失败:", error);
			alert("清除失败，请重试。");
		} finally {
			setIsClearing(false);
		}
	}, [confirmInput]);

	// 取消清除操作
	const handleCancelClear = useCallback(() => {
		setShowConfirmDialog(false);
		setConfirmInput("");
	}, []);

	// 返回处理
	const handleBack = useCallback(() => {
		if (canGoBack) {
			goBack();
		}
	}, [canGoBack, goBack]);

	return (
		<div className="h-full flex flex-col">
			{/* 确认对话框 */}
			{showConfirmDialog && (
				<div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
					<div className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4">
						<div className="flex items-start mb-4">
							<AlertCircle className="h-6 w-6 text-red-600 dark:text-red-400 mr-3 mt-0.5 flex-shrink-0" />
							<div>
								<h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
									确认删除所有数据
								</h3>
								<p className="text-sm text-gray-600 dark:text-gray-300 mb-4">
									此操作将永久删除所有数据，包括任务、分类、财务记录、日记和统计数据。
								</p>
								<p className="text-sm text-red-600 dark:text-red-400 mb-4">
									<strong>此操作不可恢复！</strong>
								</p>
								<p className="text-sm text-gray-600 dark:text-gray-300 mb-4">
									请输入 <strong>DELETE</strong> 来确认删除：
								</p>
								<input
									type="text"
									value={confirmInput}
									onChange={(e) => setConfirmInput(e.target.value)}
									placeholder="输入 DELETE"
									className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md 
										focus:outline-none focus:ring-2 focus:ring-red-500 focus:border-transparent
										bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
									autoFocus
								/>
							</div>
						</div>
						<div className="flex justify-end space-x-3">
							<button
								onClick={handleCancelClear}
								className="px-4 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 
									dark:hover:text-white transition-colors"
							>
								取消
							</button>
							<button
								onClick={handleConfirmClear}
								disabled={confirmInput !== "DELETE"}
								className={`px-4 py-2 rounded-md font-medium text-white transition-colors ${
									confirmInput === "DELETE"
										? "bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500"
										: "bg-gray-400 cursor-not-allowed"
								}`}
							>
								确认删除
							</button>
						</div>
					</div>
				</div>
			)}

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
							数据清理
						</h1>
					</div>
				</div>
			</div>

			{/* 可滚动内容区域 */}
			<div className="flex-1 overflow-y-auto py-4 px-4 md:px-6 scroll-container">
				<div className="max-w-2xl mx-auto space-y-6">
					{/* 危险操作警告 */}
					<div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6">
						<div className="flex items-start">
							<AlertCircle className="h-6 w-6 text-red-600 dark:text-red-400 mr-3 mt-0.5 flex-shrink-0" />
							<div>
								<h2 className="text-lg font-semibold text-red-900 dark:text-red-100 mb-3">
									危险操作警告
								</h2>
								<div className="text-sm text-red-700 dark:text-red-300 space-y-2">
									<p>
										<strong>此操作将永久删除所有数据，包括：</strong>
									</p>
									<ul className="list-disc list-inside space-y-1 ml-4">
										<li>所有任务和计时记录</li>
										<li>所有分类信息</li>
										<li>所有财务记录和账单</li>
										<li>所有日记和笔记</li>
										<li>所有统计数据和历史记录</li>
										<li>应用的所有配置和设置</li>
									</ul>
									<p>
										<strong>操作不可恢复，请谨慎使用。</strong>
									</p>
								</div>
							</div>
						</div>
					</div>

					{/* 建议备份 */}
					<div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-6">
						<div className="flex items-start">
							<Database className="h-6 w-6 text-blue-600 dark:text-blue-400 mr-3 mt-0.5 flex-shrink-0" />
							<div>
								<h3 className="text-lg font-semibold text-blue-900 dark:text-blue-100 mb-3">
									建议先备份数据
								</h3>
								<div className="text-sm text-blue-700 dark:text-blue-300 space-y-2">
									<p>在执行数据清理之前，强烈建议您：</p>
									<ul className="list-disc list-inside space-y-1 ml-4">
										<li>
											使用<strong>备份与恢复</strong>功能创建数据备份
										</li>
										<li>
											或使用<strong>数据导出</strong>功能导出重要数据
										</li>
										<li>确保备份文件已保存到安全的位置</li>
									</ul>
									<p>这样可以在需要时恢复您的数据。</p>
								</div>
							</div>
						</div>
					</div>

					{/* 使用场景说明 */}
					<div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-6">
						<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">
							适用场景
						</h3>
						<div className="text-sm text-gray-700 dark:text-gray-300 space-y-2">
							<p>数据清理功能适用于以下场景：</p>
							<ul className="list-disc list-inside space-y-1 ml-4">
								<li>重新开始使用应用，希望从零开始记录</li>
								<li>测试完毕，需要清除测试数据</li>
								<li>数据出现问题，需要重置应用状态</li>
								<li>转移应用到其他设备前的数据清理</li>
							</ul>
						</div>
					</div>

					{/* 清理操作 */}
					<div className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6">
						<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
							执行数据清理
						</h3>
						<div className="space-y-4">
							<div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
								<div className="flex items-center">
									<AlertCircle className="h-5 w-5 text-red-600 dark:text-red-400 mr-2" />
									<span className="text-sm text-red-700 dark:text-red-300">
										点击下方按钮将要求您输入"DELETE"来确认清除所有数据
									</span>
								</div>
							</div>

							<button
								onClick={handleClearData}
								disabled={isClearing}
								className={`w-full flex items-center justify-center px-6 py-3 rounded-lg font-medium text-white transition-colors ${
									isClearing
										? "bg-gray-400 cursor-not-allowed"
										: "bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2"
								}`}
							>
								<Trash2 className="h-5 w-5 mr-2" />
								{isClearing ? "正在清理..." : "清除所有数据"}
							</button>
						</div>
					</div>

					{/* 清理后的说明 */}
					<div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-6">
						<h3 className="text-lg font-semibold text-yellow-900 dark:text-yellow-100 mb-3">
							清理后的状态
						</h3>
						<div className="text-sm text-yellow-700 dark:text-yellow-300 space-y-2">
							<p>数据清理完成后，应用将：</p>
							<ul className="list-disc list-inside space-y-1 ml-4">
								<li>回到全新安装的状态</li>
								<li>所有数据表将被重置</li>
								<li>应用配置将恢复默认设置</li>
								<li>需要重新配置各项功能</li>
							</ul>
							<p>
								<strong>建议清理后重启应用以确保完全生效。</strong>
							</p>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
}
