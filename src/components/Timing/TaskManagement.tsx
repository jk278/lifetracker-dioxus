import { invoke } from "@tauri-apps/api/core";
import {
	ChevronDown,
	ChevronUp,
	Edit,
	Plus,
	Tag,
	Trash2,
	User,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useDataRefresh } from "../../hooks/useDataRefresh";
import { AnimatedList, InteractiveButton } from "../Animation";

interface Task {
	id: string;
	name: string;
	description?: string;
	category_id?: string;
	category_name?: string;
	duration_seconds: number;
	tags: string[];
	created_at: string;
	updated_at: string;
}

interface Category {
	id: string;
	name: string;
	color: string;
	icon: string;
}

export function TaskManagement() {
	const [tasks, setTasks] = useState<Task[]>([]);
	const [categories, setCategories] = useState<Category[]>([]);
	const [loading, setLoading] = useState(false);
	const [showCreateForm, setShowCreateForm] = useState(false);
	const [editingTask, setEditingTask] = useState<Task | null>(null);
	const [expandedTasks, setExpandedTasks] = useState<Set<string>>(new Set());

	const [newTask, setNewTask] = useState({
		name: "",
		description: "",
		category_id: "",
		tags: "",
	});

	// 获取任务列表
	const fetchTasks = useCallback(async () => {
		setLoading(true);
		try {
			const result = await invoke<Task[]>("get_tasks");
			setTasks(result);
		} catch (error) {
			console.error("获取任务失败:", error);
		} finally {
			setLoading(false);
		}
	}, []);

	// 获取分类列表
	const fetchCategories = useCallback(async () => {
		try {
			const result = await invoke<Category[]>("get_categories");
			setCategories(result);
		} catch (error) {
			console.error("获取分类失败:", error);
		}
	}, []);

	// 刷新所有数据
	const refreshAllData = useCallback(async () => {
		await Promise.all([fetchTasks(), fetchCategories()]);
	}, [fetchTasks, fetchCategories]);

	// 设置数据刷新监听
	useDataRefresh(refreshAllData, {
		onRefresh: (changeType) => {
			console.log(`任务管理页面收到数据变化通知: ${changeType}`);
		},
	});

	// 初始化数据获取
	useEffect(() => {
		refreshAllData();
	}, [refreshAllData]);

	// 创建任务
	const handleCreateTask = useCallback(async () => {
		if (!newTask.name.trim()) return;

		try {
			const taskData = {
				name: newTask.name.trim(),
				description: newTask.description.trim() || null,
				category_id: newTask.category_id || null,
				tags: newTask.tags
					? newTask.tags.split(",").map((tag) => tag.trim())
					: [],
			};

			await invoke("create_task", { taskData });
			setNewTask({ name: "", description: "", category_id: "", tags: "" });
			setShowCreateForm(false);
			await fetchTasks(); // 刷新任务列表
		} catch (error) {
			console.error("创建任务失败:", error);
			alert("创建任务失败，请重试");
		}
	}, [newTask, fetchTasks]);

	// 删除任务
	const handleDeleteTask = useCallback(
		async (taskId: string) => {
			if (!confirm("确定要删除这个任务吗？")) return;

			try {
				await invoke("delete_task", { taskId });
				await fetchTasks(); // 刷新任务列表
			} catch (error) {
				console.error("删除任务失败:", error);
				alert("删除任务失败，请重试");
			}
		},
		[fetchTasks],
	);

	// 更新任务
	const handleUpdateTask = useCallback(
		async (taskId: string, updates: any) => {
			try {
				await invoke("update_task", { taskId, updates });
				setEditingTask(null);
				await fetchTasks(); // 刷新任务列表
			} catch (error) {
				console.error("更新任务失败:", error);
				alert("更新任务失败，请重试");
			}
		},
		[fetchTasks],
	);

	// 切换任务展开状态
	const toggleTaskExpanded = useCallback((taskId: string) => {
		setExpandedTasks((prev) => {
			const newSet = new Set(prev);
			if (newSet.has(taskId)) {
				newSet.delete(taskId);
			} else {
				newSet.add(taskId);
			}
			return newSet;
		});
	}, []);

	// 格式化时长
	const formatDuration = (seconds: number) => {
		const hours = Math.floor(seconds / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		return `${hours}h ${minutes}m`;
	};

	// 获取分类信息
	const getCategoryInfo = (categoryId?: string) => {
		return categories.find((cat) => cat.id === categoryId);
	};

	return (
		<div className="space-y-6">
			{/* 页面标题和操作 */}
			<div className="flex items-center justify-between">
				<h2 className="text-2xl font-bold text-gray-900 dark:text-white">
					任务管理
				</h2>
				<InteractiveButton
					onClick={() => setShowCreateForm(true)}
					variant="primary"
					className="flex items-center space-x-2 px-4 py-2 bg-theme-primary text-white rounded-lg hover:bg-theme-primary-hover theme-transition"
				>
					<Plus className="h-4 w-4" />
					<span>新建任务</span>
				</InteractiveButton>
			</div>

			{/* 创建任务表单 */}
			{showCreateForm && (
				<div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
					<h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
						创建新任务
					</h3>
					<div className="space-y-4">
						<div>
							<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
								任务名称 *
							</label>
							<input
								type="text"
								value={newTask.name}
								onChange={(e) =>
									setNewTask((prev) => ({ ...prev, name: e.target.value }))
								}
								className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
								placeholder="输入任务名称"
							/>
						</div>
						<div>
							<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
								任务描述
							</label>
							<textarea
								value={newTask.description}
								onChange={(e) =>
									setNewTask((prev) => ({
										...prev,
										description: e.target.value,
									}))
								}
								className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
								placeholder="输入任务描述"
								rows={3}
							/>
						</div>
						<div>
							<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
								分类
							</label>
							<select
								value={newTask.category_id}
								onChange={(e) =>
									setNewTask((prev) => ({
										...prev,
										category_id: e.target.value,
									}))
								}
								className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
							>
								<option value="">选择分类</option>
								{categories.map((category) => (
									<option key={category.id} value={category.id}>
										{category.name}
									</option>
								))}
							</select>
						</div>
						<div>
							<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
								标签 (用逗号分隔)
							</label>
							<input
								type="text"
								value={newTask.tags}
								onChange={(e) =>
									setNewTask((prev) => ({ ...prev, tags: e.target.value }))
								}
								className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
								placeholder="例如: 工作, 重要"
							/>
						</div>
						<div className="flex space-x-3">
							<button
								onClick={handleCreateTask}
								className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
							>
								创建任务
							</button>
							<button
								onClick={() => {
									setShowCreateForm(false);
									setNewTask({
										name: "",
										description: "",
										category_id: "",
										tags: "",
									});
								}}
								className="px-4 py-2 bg-gray-500 text-white rounded-md hover:bg-gray-600 transition-colors"
							>
								取消
							</button>
						</div>
					</div>
				</div>
			)}

			{/* 任务列表 */}
			{loading ? (
				<div className="flex justify-center py-8">
					<div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
				</div>
			) : tasks.length === 0 ? (
				<div className="text-center py-8 text-gray-500 dark:text-gray-400">
					暂无任务，点击"新建任务"开始添加
				</div>
			) : (
				<AnimatedList
					animation="slide"
					staggerDelay={0.1}
					className="space-y-4"
				>
					{tasks.map((task) => {
						const isExpanded = expandedTasks.has(task.id);
						const categoryInfo = getCategoryInfo(task.category_id);

						return (
							<div
								key={task.id}
								className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4"
							>
								{/* 任务基本信息 */}
								<div className="flex items-center justify-between">
									<div className="flex items-center space-x-3 flex-1">
										<InteractiveButton
											onClick={() => toggleTaskExpanded(task.id)}
											variant="ghost"
											size="sm"
											className="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
										>
											{isExpanded ? (
												<ChevronUp className="h-4 w-4" />
											) : (
												<ChevronDown className="h-4 w-4" />
											)}
										</InteractiveButton>
										<div className="flex-1">
											<h3 className="font-medium text-gray-900 dark:text-white">
												{task.name}
											</h3>
											<div className="flex items-center space-x-4 mt-1 text-sm text-gray-500 dark:text-gray-400">
												{categoryInfo && (
													<span className="flex items-center space-x-1">
														<span
															className="w-2 h-2 rounded-full"
															style={{ backgroundColor: categoryInfo.color }}
														/>
														<span>{categoryInfo.name}</span>
													</span>
												)}
												<span className="flex items-center space-x-1">
													<User className="h-3 w-3" />
													<span>{formatDuration(task.duration_seconds)}</span>
												</span>
												{task.tags.length > 0 && (
													<span className="flex items-center space-x-1">
														<Tag className="h-3 w-3" />
														<span>{task.tags.join(", ")}</span>
													</span>
												)}
											</div>
										</div>
									</div>
									<div className="flex items-center space-x-2">
										<InteractiveButton
											onClick={() => setEditingTask(task)}
											variant="ghost"
											size="sm"
											className="p-2 text-gray-400 hover:text-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded"
										>
											<Edit className="h-4 w-4" />
										</InteractiveButton>
										<InteractiveButton
											onClick={() => handleDeleteTask(task.id)}
											variant="ghost"
											size="sm"
											className="p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded"
										>
											<Trash2 className="h-4 w-4" />
										</InteractiveButton>
									</div>
								</div>

								{/* 展开的详细信息 */}
								{isExpanded && (
									<div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
										<div className="grid grid-cols-2 gap-4 text-sm">
											<div>
												<span className="font-medium text-gray-700 dark:text-gray-300">
													创建时间:
												</span>
												<span className="ml-2 text-gray-500 dark:text-gray-400">
													{new Date(task.created_at).toLocaleString()}
												</span>
											</div>
											<div>
												<span className="font-medium text-gray-700 dark:text-gray-300">
													更新时间:
												</span>
												<span className="ml-2 text-gray-500 dark:text-gray-400">
													{new Date(task.updated_at).toLocaleString()}
												</span>
											</div>
										</div>
										{task.description && (
											<div className="mt-3">
												<span className="font-medium text-gray-700 dark:text-gray-300">
													描述:
												</span>
												<p className="mt-1 text-gray-600 dark:text-gray-400">
													{task.description}
												</p>
											</div>
										)}
									</div>
								)}
							</div>
						);
					})}
				</AnimatedList>
			)}

			{/* 编辑任务模态框 */}
			{editingTask && (
				<div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
					<div className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4">
						<h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
							编辑任务
						</h3>
						<div className="space-y-4">
							<div>
								<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
									任务名称
								</label>
								<input
									type="text"
									value={editingTask.name}
									onChange={(e) =>
										setEditingTask((prev) =>
											prev ? { ...prev, name: e.target.value } : null,
										)
									}
									className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
								/>
							</div>
							<div>
								<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
									任务描述
								</label>
								<textarea
									value={editingTask.description || ""}
									onChange={(e) =>
										setEditingTask((prev) =>
											prev ? { ...prev, description: e.target.value } : null,
										)
									}
									className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
									rows={3}
								/>
							</div>
							<div className="flex space-x-3">
								<button
									onClick={() =>
										handleUpdateTask(editingTask.id, {
											name: editingTask.name,
											description: editingTask.description,
										})
									}
									className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
								>
									保存
								</button>
								<button
									onClick={() => setEditingTask(null)}
									className="px-4 py-2 bg-gray-500 text-white rounded-md hover:bg-gray-600 transition-colors"
								>
									取消
								</button>
							</div>
						</div>
					</div>
				</div>
			)}
		</div>
	);
}
