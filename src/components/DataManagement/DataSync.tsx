import { invoke } from "@tauri-apps/api/core";
import {
	AlertTriangle,
	ArrowLeft,
	Cloud,
	GitMerge,
	Info,
	RefreshCw,
	Save,
	Wifi,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useNavigation } from "../../hooks/useRouter";
import { ConflictResolution } from "./ConflictResolution";

// 与 ConflictResolution 保持一致的冲突项接口（简化）
interface ConflictItem {
	id: string;
	name: string;
	local_modified: string;
	remote_modified?: string;
	conflict_type: string;
	local_preview: any;
	remote_preview: any;
	file_size: number;
	local_hash: string;
	remote_hash?: string;
}

export function DataSync() {
	const { canGoBack, goBack } = useNavigation();

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
	const [showConflictResolution, setShowConflictResolution] = useState(false);
	const [conflictCount, setConflictCount] = useState(0);
	const [pendingConflicts, setPendingConflicts] = useState<ConflictItem[]>([]);

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

	// 检查待解决冲突
	const checkConflicts = useCallback(async () => {
		try {
			const conflicts = (await invoke(
				"get_pending_conflicts",
			)) as ConflictItem[];
			setPendingConflicts(conflicts);
			setConflictCount(conflicts.length);
		} catch (error) {
			console.error("检查冲突失败:", error);
		}
	}, []);

	// 初始化
	useEffect(() => {
		fetchSyncConfig();
		fetchSyncStatus();
		checkConflicts();
	}, [fetchSyncConfig, fetchSyncStatus, checkConflicts]);

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
			return;
		}

		setIsLoading(true);
		try {
			const result = await invoke("start_sync");

			// 检查结果是否包含冲突信息
			if (typeof result === "string" && result.includes("冲突需要解决")) {
				// 同步检测到冲突，立即检查冲突状态
				await checkConflicts();
				alert(`${result}`);
				// 显示冲突解决界面
				setShowConflictResolution(true);
			} else {
				// 无冲突，正常完成
				alert(`同步成功：${result}`);
			}

			// 刷新状态
			await fetchSyncStatus();

			// 再次检查冲突（确保状态同步）
			await checkConflicts();
		} catch (error) {
			console.error("同步失败:", error);
			alert(`同步失败：${error}`);
		} finally {
			setIsLoading(false);
		}
	}, [syncConfig.enabled, fetchSyncStatus, checkConflicts]);

	// 不再自动显示冲突解决界面，由用户主动触发

	// 处理冲突解决完成
	const handleConflictResolutionComplete = useCallback(() => {
		setShowConflictResolution(false);
		setConflictCount(0);
		setPendingConflicts([]);
		fetchSyncStatus();
	}, [fetchSyncStatus]);

	// 返回处理
	const handleBack = useCallback(() => {
		if (canGoBack) {
			goBack();
		}
	}, [canGoBack, goBack]);

	// 如果显示冲突解决界面，渲染冲突解决组件
	if (showConflictResolution) {
		return (
			<ConflictResolution
				conflicts={pendingConflicts}
				onResolutionComplete={handleConflictResolutionComplete}
			/>
		);
	}

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
							多端同步
						</h1>
					</div>
				</div>
			</div>

			{/* 可滚动内容区域 */}
			<div className="flex-1 overflow-y-auto py-4 px-4 md:px-6 scroll-container">
				<div className="max-w-2xl mx-auto space-y-6">
					{/* 同步状态 */}
					<div className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6">
						<div className="flex items-center mb-4">
							<Cloud className="h-5 w-5 text-indigo-600 dark:text-indigo-400 mr-2" />
							<h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
								同步状态
							</h2>
							<button
								onClick={fetchSyncStatus}
								className="ml-auto p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
								title="刷新状态"
							>
								<RefreshCw className="h-4 w-4" />
							</button>
						</div>
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

						{/* 冲突提示 */}
						{conflictCount > 0 && (
							<div className="mt-3 p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-md">
								<div className="flex items-center justify-between">
									<div className="flex items-center">
										<AlertTriangle className="h-4 w-4 text-amber-600 dark:text-amber-400 mr-2" />
										<div className="text-sm text-amber-700 dark:text-amber-300">
											检测到 {conflictCount} 个同步冲突需要解决
										</div>
									</div>
									<button
										onClick={() => setShowConflictResolution(true)}
										className="flex items-center px-3 py-1 text-xs font-medium text-amber-800 dark:text-amber-200 bg-amber-100 dark:bg-amber-800/30 hover:bg-amber-200 dark:hover:bg-amber-800/50 rounded-md transition-colors"
									>
										<GitMerge className="h-3 w-3 mr-1" />
										解决冲突
									</button>
								</div>
							</div>
						)}
					</div>

					{/* 基本配置 */}
					<div className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6">
						<h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
							基本配置
						</h2>

						<div className="space-y-4">
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
					</div>

					{/* WebDAV 配置 - 只在启用同步时显示 */}
					{syncConfig.enabled && syncConfig.provider === "webdav" && (
						<div className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6">
							<h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
								WebDAV 配置
							</h2>

							<div className="space-y-4">
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
										className={`flex items-center px-4 py-2 rounded-md font-medium text-white transition-colors ${
											isTesting
												? "bg-gray-400 cursor-not-allowed"
												: "bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
										}`}
									>
										<Wifi className="h-4 w-4 mr-2" />
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
						</div>
					)}

					{/* 同步未启用时的提示 */}
					{!syncConfig.enabled && (
						<div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-6">
							<div className="flex items-center">
								<Info className="h-5 w-5 text-gray-400 mr-2" />
								<span className="text-sm text-gray-600 dark:text-gray-400">
									请先启用多端同步功能以配置同步设置
								</span>
							</div>
						</div>
					)}

					{/* 操作按钮 */}
					<div className="flex flex-col sm:flex-row gap-4">
						<button
							onClick={handleSaveConfig}
							disabled={isLoading}
							className={`flex-1 flex items-center justify-center px-6 py-2 rounded-md font-medium text-white transition-colors ${
								isLoading
									? "bg-gray-400 cursor-not-allowed"
									: "bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500"
							}`}
						>
							<Save className="h-4 w-4 mr-2" />
							{isLoading ? "保存中..." : "保存配置"}
						</button>

						<button
							onClick={handleStartSync}
							disabled={isLoading || !syncConfig.enabled}
							className={`flex-1 flex items-center justify-center px-6 py-2 rounded-md font-medium text-white transition-colors ${
								isLoading || !syncConfig.enabled
									? "bg-gray-400 cursor-not-allowed"
									: "bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500"
							}`}
						>
							<Cloud className="h-4 w-4 mr-2" />
							{syncStatus.is_syncing
								? "同步中..."
								: !syncConfig.enabled
									? "请先启用同步"
									: "立即同步"}
						</button>
					</div>

					{/* 使用说明 */}
					<div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-6">
						<div className="flex items-start">
							<Info className="h-4 w-4 text-blue-600 dark:text-blue-400 mr-2 mt-0.5 flex-shrink-0" />
							<div className="text-sm text-blue-700 dark:text-blue-300">
								<p className="font-medium mb-2">同步功能说明：</p>
								<ul className="list-disc list-inside space-y-1">
									<li>
										支持 WebDAV 协议的云存储服务（如 Nextcloud、ownCloud）
									</li>
									<li>密码会进行加密存储，确保安全性</li>
									<li>冲突解决：手动处理可让您选择保留哪个版本的数据</li>
									<li>建议首次同步前先备份本地数据</li>
								</ul>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
}
