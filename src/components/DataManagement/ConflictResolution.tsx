import { invoke } from "@tauri-apps/api/core";
import {
	AlertTriangle,
	ArrowLeft,
	CheckCircle,
	Clock,
	Database,
	Download,
	GitMerge,
	Upload,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useNavigation } from "../../hooks/useRouter";

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

interface ConflictResolutionProps {
	conflicts?: ConflictItem[];
	onResolutionComplete?: () => void;
}

export function ConflictResolution({
	conflicts: propConflicts,
	onResolutionComplete,
}: ConflictResolutionProps) {
	const { canGoBack, goBack } = useNavigation();

	// 冲突列表
	const [conflicts, setConflicts] = useState<ConflictItem[]>(
		propConflicts || [],
	);
	// 加载状态：如果 props 未提供冲突列表，则默认为加载中
	const [isLoading, setIsLoading] = useState(!propConflicts);
	const [selectedResolutions, setSelectedResolutions] = useState<
		Record<string, string>
	>({});
	const [isResolving, setIsResolving] = useState(false);
	const [currentConflictIndex, setCurrentConflictIndex] = useState(0);
	const [resolutionResult, setResolutionResult] = useState<string>("");

	// 获取待解决的冲突列表
	const fetchConflicts = useCallback(async () => {
		if (propConflicts) {
			// 如果通过 props 已经传入冲突列表，则不需要再加载
			setIsLoading(false);
			return;
		}

		try {
			setIsLoading(true);
			const result = await invoke<ConflictItem[]>("get_pending_conflicts");
			setConflicts(result);
		} catch (error) {
			console.error("获取冲突列表失败:", error);
		} finally {
			setIsLoading(false);
		}
	}, [propConflicts]);

	useEffect(() => {
		fetchConflicts();
	}, [fetchConflicts]);

	// 选择解决方案
	const handleResolutionSelect = useCallback(
		(conflictId: string, resolution: string) => {
			setSelectedResolutions((prev) => ({
				...prev,
				[conflictId]: resolution,
			}));
		},
		[],
	);

	// 应用解决方案
	const handleApplyResolution = useCallback(async () => {
		setIsResolving(true);
		try {
			const result = await invoke<string>("resolve_conflicts", {
				resolutions: selectedResolutions,
			});
			setResolutionResult(result);
			onResolutionComplete?.();
		} catch (error) {
			console.error("解决冲突失败:", error);
			setResolutionResult(`解决冲突失败: ${error}`);
		} finally {
			setIsResolving(false);
		}
	}, [selectedResolutions, onResolutionComplete]);

	// 返回处理
	const handleBack = useCallback(() => {
		if (canGoBack) {
			goBack();
		}
	}, [canGoBack, goBack]);

	const currentConflict = conflicts[currentConflictIndex];
	const hasNextConflict = currentConflictIndex < conflicts.length - 1;
	const hasPrevConflict = currentConflictIndex > 0;

	// 加载中时显示指示器
	if (isLoading) {
		return (
			<div className="h-full flex flex-col items-center justify-center">
				<div className="flex flex-col items-center space-y-4">
					<div className="animate-spin rounded-full h-10 w-10 border-b-2 border-indigo-600" />
					<span className="text-sm text-gray-600 dark:text-gray-400">
						加载冲突列表...
					</span>
				</div>
			</div>
		);
	}

	// 如果没有冲突，显示无冲突状态
	if (conflicts.length === 0) {
		return (
			<div className="h-full flex flex-col">
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
								冲突解决
							</h1>
						</div>
					</div>
				</div>

				<div className="flex-1 flex items-center justify-center">
					<div className="text-center">
						<CheckCircle className="w-16 h-16 mx-auto text-green-500 mb-4" />
						<h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
							没有需要解决的冲突
						</h2>
						<p className="text-gray-600 dark:text-gray-400">
							当前没有检测到数据同步冲突
						</p>
					</div>
				</div>
			</div>
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
							解决同步冲突
						</h1>
					</div>
					<div className="text-sm text-gray-600 dark:text-gray-400">
						{currentConflictIndex + 1} / {conflicts.length}
					</div>
				</div>
			</div>

			{/* 可滚动内容区域 */}
			<div className="flex-1 overflow-y-auto py-4 px-4 md:px-6 scroll-container">
				<div className="max-w-4xl mx-auto space-y-6">
					{/* 冲突概览 */}
					<div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg p-4">
						<div className="flex items-start">
							<AlertTriangle className="h-5 w-5 text-amber-600 dark:text-amber-400 mr-3 mt-0.5" />
							<div>
								<h3 className="font-medium text-amber-800 dark:text-amber-200">
									检测到数据同步冲突
								</h3>
								<p className="text-sm text-amber-700 dark:text-amber-300 mt-1">
									本地数据与远程数据存在差异，请选择如何处理这些冲突。
								</p>
							</div>
						</div>
					</div>

					{/* 当前冲突详情 */}
					{currentConflict && (
						<div className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6">
							<div className="flex items-center mb-4">
								<Database className="h-5 w-5 text-indigo-600 dark:text-indigo-400 mr-2" />
								<h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
									{currentConflict.name}
								</h2>
								<span className="ml-2 px-2 py-1 text-xs font-medium bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300 rounded-full">
									{currentConflict.conflict_type === "fresh_data"
										? "新数据冲突"
										: currentConflict.conflict_type === "timestamp"
											? "时间戳冲突"
											: "内容冲突"}
								</span>
							</div>

							{/* 时间信息 */}
							<div className="grid grid-cols-2 gap-4 mb-6">
								<div className="flex items-center text-sm text-gray-600 dark:text-gray-400">
									<Clock className="h-4 w-4 mr-2" />
									<span>
										本地修改:{" "}
										{new Date(currentConflict.local_modified).toLocaleString()}
									</span>
								</div>
								{currentConflict.remote_modified && (
									<div className="flex items-center text-sm text-gray-600 dark:text-gray-400">
										<Clock className="h-4 w-4 mr-2" />
										<span>
											远程修改:{" "}
											{new Date(
												currentConflict.remote_modified,
											).toLocaleString()}
										</span>
									</div>
								)}
							</div>

							{/* 解决方案选择 */}
							<div className="space-y-4">
								<h3 className="font-medium text-gray-900 dark:text-gray-100">
									请选择解决方案：
								</h3>

								<div className="grid gap-4">
									{/* 智能合并 */}
									<label className="cursor-pointer">
										<input
											type="radio"
											name={`resolution-${currentConflict.id}`}
											value="merge"
											checked={
												selectedResolutions[currentConflict.id] === "merge"
											}
											onChange={(e) =>
												handleResolutionSelect(
													currentConflict.id,
													e.target.value,
												)
											}
											className="sr-only"
										/>
										<div
											className={`p-4 border rounded-lg transition-colors ${
												selectedResolutions[currentConflict.id] === "merge"
													? "border-indigo-500 bg-indigo-50 dark:bg-indigo-900/20"
													: "border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
											}`}
										>
											<div className="flex items-start">
												<GitMerge className="h-5 w-5 text-indigo-600 dark:text-indigo-400 mr-3 mt-0.5" />
												<div>
													<h4 className="font-medium text-gray-900 dark:text-gray-100">
														智能合并 (推荐)
													</h4>
													<p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
														自动合并本地和远程数据，去除重复项，保留所有有效数据
													</p>
												</div>
											</div>
										</div>
									</label>

									{/* 使用本地数据 */}
									<label className="cursor-pointer">
										<input
											type="radio"
											name={`resolution-${currentConflict.id}`}
											value="use_local"
											checked={
												selectedResolutions[currentConflict.id] === "use_local"
											}
											onChange={(e) =>
												handleResolutionSelect(
													currentConflict.id,
													e.target.value,
												)
											}
											className="sr-only"
										/>
										<div
											className={`p-4 border rounded-lg transition-colors ${
												selectedResolutions[currentConflict.id] === "use_local"
													? "border-green-500 bg-green-50 dark:bg-green-900/20"
													: "border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
											}`}
										>
											<div className="flex items-start">
												<Upload className="h-5 w-5 text-green-600 dark:text-green-400 mr-3 mt-0.5" />
												<div>
													<h4 className="font-medium text-gray-900 dark:text-gray-100">
														使用本地数据
													</h4>
													<p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
														保留本地数据，覆盖远程数据
													</p>
												</div>
											</div>
										</div>
									</label>

									{/* 使用远程数据 */}
									<label className="cursor-pointer">
										<input
											type="radio"
											name={`resolution-${currentConflict.id}`}
											value="use_remote"
											checked={
												selectedResolutions[currentConflict.id] === "use_remote"
											}
											onChange={(e) =>
												handleResolutionSelect(
													currentConflict.id,
													e.target.value,
												)
											}
											className="sr-only"
										/>
										<div
											className={`p-4 border rounded-lg transition-colors ${
												selectedResolutions[currentConflict.id] === "use_remote"
													? "border-blue-500 bg-blue-50 dark:bg-blue-900/20"
													: "border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
											}`}
										>
											<div className="flex items-start">
												<Download className="h-5 w-5 text-blue-600 dark:text-blue-400 mr-3 mt-0.5" />
												<div>
													<h4 className="font-medium text-gray-900 dark:text-gray-100">
														使用远程数据
													</h4>
													<p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
														下载远程数据，覆盖本地数据
													</p>
												</div>
											</div>
										</div>
									</label>
								</div>
							</div>
						</div>
					)}

					{/* 导航和应用按钮 */}
					<div className="flex justify-between items-center">
						<div className="flex space-x-2">
							<button
								onClick={() => setCurrentConflictIndex((prev) => prev - 1)}
								disabled={!hasPrevConflict}
								className="px-3 py-2 text-sm text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
							>
								上一个
							</button>
							<button
								onClick={() => setCurrentConflictIndex((prev) => prev + 1)}
								disabled={!hasNextConflict}
								className="px-3 py-2 text-sm text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
							>
								下一个
							</button>
						</div>

						<button
							onClick={handleApplyResolution}
							disabled={
								isResolving ||
								Object.keys(selectedResolutions).length !== conflicts.length
							}
							className={`px-6 py-2 rounded-md font-medium text-white theme-transition ${
								isResolving ||
								Object.keys(selectedResolutions).length !== conflicts.length
									? "bg-gray-400 cursor-not-allowed"
									: "bg-theme-primary hover:bg-theme-primary-hover"
							}`}
						>
							{isResolving ? "解决中..." : "应用解决方案"}
						</button>
					</div>

					{/* 结果显示 */}
					{resolutionResult && (
						<div
							className={`p-4 rounded-lg ${
								resolutionResult.includes("成功")
									? "bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 text-green-700 dark:text-green-300"
									: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-300"
							}`}
						>
							{resolutionResult}
						</div>
					)}
				</div>
			</div>
		</div>
	);
}
