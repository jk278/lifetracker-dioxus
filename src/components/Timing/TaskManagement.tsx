import { invoke } from "@tauri-apps/api/core";
import { Clock, Edit, Plus, Search, Tag, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import type { Category, Task } from "../../types";

interface TaskManagementProps {
	tasks: Task[];
	onTasksUpdate: () => void;
}

const TaskManagement: React.FC<TaskManagementProps> = ({
	tasks,
	onTasksUpdate,
}) => {
	const [categories, setCategories] = useState<Category[]>([]);
	const [searchTerm, setSearchTerm] = useState("");
	const [selectedCategory, setSelectedCategory] = useState("");
	const [showCreateDialog, setShowCreateDialog] = useState(false);
	const [editingTask, setEditingTask] = useState<Task | null>(null);
	const [newTask, setNewTask] = useState({
		name: "",
		description: "",
		category_id: "",
		tags: "",
	});

	// 获取分类列表
	const fetchCategories = async () => {
		try {
			const categoryList = await invoke<Category[]>("get_categories");
			setCategories(categoryList);
		} catch (error) {
			console.error("获取分类列表失败:", error);
		}
	};

	// 创建任务
	const createTask = async () => {
		if (!newTask.name.trim()) return;

		try {
			console.log("TaskManagement - 创建任务开始，参数:", {
				name: newTask.name,
				description: newTask.description || null,
				category_id: newTask.category_id || null,
				tags: newTask.tags
					? newTask.tags.split(",").map((tag) => tag.trim())
					: null,
			});

			const result = await invoke("create_task", {
				request: {
					name: newTask.name,
					description: newTask.description || null,
					category_id: newTask.category_id || null,
					tags: newTask.tags
						? newTask.tags.split(",").map((tag) => tag.trim())
						: null,
				},
			});

			console.log("TaskManagement - 任务创建成功，返回结果:", result);

			setNewTask({ name: "", description: "", category_id: "", tags: "" });
			setShowCreateDialog(false);

			// 稍等一下再刷新，确保数据库操作完全完成
			setTimeout(() => {
				console.log("TaskManagement - 开始刷新任务列表");
				onTasksUpdate();
			}, 200);
		} catch (error) {
			console.error("TaskManagement - 创建任务失败:", error);
			alert(`创建任务失败: ${error}`);
		}
	};

	// 更新任务
	const updateTask = async () => {
		if (!editingTask || !newTask.name.trim()) return;

		try {
			await invoke("update_task", {
				taskId: editingTask.id,
				request: {
					name: newTask.name,
					description: newTask.description || null,
					category_id: newTask.category_id || null,
					tags: newTask.tags
						? newTask.tags.split(",").map((tag) => tag.trim())
						: null,
				},
			});

			setEditingTask(null);
			setNewTask({ name: "", description: "", category_id: "", tags: "" });
			onTasksUpdate();
		} catch (error) {
			console.error("更新任务失败:", error);
		}
	};

	// 删除任务
	const deleteTask = async (taskId: string) => {
		if (!confirm("确定要删除这个任务吗？")) return;

		try {
			await invoke("delete_task", { taskId });
			onTasksUpdate();
		} catch (error) {
			console.error("删除任务失败:", error);
		}
	};

	// 开始编辑任务
	const startEditTask = (task: Task) => {
		setEditingTask(task);
		setNewTask({
			name: task.name,
			description: task.description || "",
			category_id: task.category_id || "",
			tags: task.tags ? task.tags.join(", ") : "",
		});
		setShowCreateDialog(true);
	};

	// 筛选任务
	const filteredTasks = tasks.filter((task) => {
		const matchesSearch =
			task.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
			task.description?.toLowerCase().includes(searchTerm.toLowerCase());
		const matchesCategory =
			!selectedCategory || task.category_id === selectedCategory;
		return matchesSearch && matchesCategory;
	});

	// 格式化时间
	const formatDuration = (seconds: number): string => {
		const hours = Math.floor(seconds / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		return `${hours}h ${minutes}m`;
	};

	useEffect(() => {
		fetchCategories();
	}, []);

	return (
		<div className="space-y-6">
			{/* 页面标题和工具栏 */}
			<div className="flex items-center justify-between">
				<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
					任务管理
				</h3>
				<button
					onClick={() => {
						setEditingTask(null);
						setNewTask({
							name: "",
							description: "",
							category_id: "",
							tags: "",
						});
						setShowCreateDialog(true);
					}}
					className="flex items-center px-4 py-2 bg-theme-primary text-white rounded-lg bg-theme-primary-hover theme-transition"
				>
					<Plus className="h-4 w-4 mr-2" />
					新建任务
				</button>
			</div>

			{/* 搜索和筛选 */}
			<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-4">
				<div className="flex flex-col sm:flex-row gap-4">
					{/* 搜索框 */}
					<div className="flex-1 relative">
						<Search className="h-5 w-5 absolute left-3 top-3 text-gray-400 dark:text-gray-500" />
						<input
							type="text"
							value={searchTerm}
							onChange={(e) => setSearchTerm(e.target.value)}
							placeholder="搜索任务名称或描述..."
							className="pl-10 pr-4 py-2 w-full border border-gray-300 dark:border-gray-700 bg-surface text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 rounded-md focus:outline-none focus:ring-2 focus:ring-theme-primary theme-transition"
						/>
					</div>

					{/* 分类筛选 */}
					<div className="sm:w-48">
						<select
							value={selectedCategory}
							onChange={(e) => setSelectedCategory(e.target.value)}
							className="w-full px-3 py-2 border border-gray-300 dark:border-gray-700 bg-surface text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-theme-primary theme-transition"
						>
							<option value="">所有分类</option>
							{categories.map((category) => (
								<option key={category.id} value={category.id}>
									{category.name}
								</option>
							))}
						</select>
					</div>
				</div>
			</div>

			{/* 任务列表 */}
			<div>
				{filteredTasks.length === 0 ? (
					<div className="text-center py-12">
						<Clock className="h-12 w-12 text-gray-400 dark:text-gray-500 mx-auto mb-4" />
						<h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
							暂无任务
						</h3>
						<p className="text-gray-500 dark:text-gray-400">
							{searchTerm || selectedCategory
								? "没有符合条件的任务"
								: "创建您的第一个任务开始计时"}
						</p>
					</div>
				) : (
					<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
						{filteredTasks.map((task) => {
							const category = categories.find(
								(cat) => cat.id === task.category_id,
							);

							return (
								<div
									key={task.id}
									className="border border-gray-200 dark:border-gray-700 bg-surface rounded-lg p-4 hover:shadow-md dark:hover:shadow-gray-700/30 transition-shadow"
								>
									<div className="flex items-start justify-between mb-3">
										<h3 className="text-lg font-semibold text-gray-900 dark:text-white truncate">
											{task.name}
										</h3>
										<div className="flex space-x-2 ml-2">
											<button
												onClick={() => startEditTask(task)}
												className="text-theme-primary hover:text-theme-primary-hover theme-transition"
											>
												<Edit className="h-4 w-4" />
											</button>
											<button
												onClick={() => deleteTask(task.id)}
												className="text-red-600 dark:text-red-400 hover:text-red-900 dark:hover:text-red-300 transition-colors"
											>
												<Trash2 className="h-4 w-4" />
											</button>
										</div>
									</div>

									{task.description && (
										<p className="text-sm text-gray-600 dark:text-gray-300 mb-3 line-clamp-2">
											{task.description}
										</p>
									)}

									<div className="flex items-center justify-between mb-3">
										{category ? (
											<span
												className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border border-gray-200 dark:border-gray-700"
												style={{
													backgroundColor: category.color + "20",
													color: category.color,
												}}
											>
												{category.name}
											</span>
										) : (
											<span className="text-xs text-gray-500 dark:text-gray-400">
												未分类
											</span>
										)}

										<span className="text-sm font-medium text-gray-900 dark:text-white">
											{formatDuration(task.duration_seconds)}
										</span>
									</div>

									{task.tags && task.tags.length > 0 && (
										<div className="flex flex-wrap gap-1">
											{task.tags.slice(0, 3).map((tag: string) => (
												<span
													key={tag}
													className="inline-flex items-center px-2 py-1 rounded-md text-xs font-medium bg-gray-100 dark:bg-gray-600 text-gray-800 dark:text-gray-200"
												>
													<Tag className="h-3 w-3 mr-1" />
													{tag}
												</span>
											))}
											{task.tags.length > 3 && (
												<span className="text-xs text-gray-500 dark:text-gray-400">
													+{task.tags.length - 3}
												</span>
											)}
										</div>
									)}

									<div className="mt-3 pt-3 border-t border-gray-100 dark:border-gray-600 text-xs text-gray-500 dark:text-gray-400">
										创建于 {new Date(task.created_at).toLocaleDateString()}
									</div>
								</div>
							);
						})}
					</div>
				)}
			</div>

			{/* 创建/编辑任务对话框 */}
			{showCreateDialog && (
				<div className="fixed inset-0 bg-black/50 dark:bg-black/70 flex items-center justify-center z-50 !mt-0">
					<div className="bg-surface rounded-lg p-6 w-full max-w-md mx-4 shadow-xl">
						<h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
							{editingTask ? "编辑任务" : "创建新任务"}
						</h3>

						<div className="space-y-4">
							<div>
								<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
									任务名称 *
								</label>
								<input
									type="text"
									value={newTask.name}
									onChange={(e) =>
										setNewTask({ ...newTask, name: e.target.value })
									}
									className="w-full px-3 py-2 border border-gray-300 dark:border-gray-700 bg-surface text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-theme-primary theme-transition"
									placeholder="输入任务名称..."
									autoFocus
								/>
							</div>

							<div>
								<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
									任务描述
								</label>
								<textarea
									value={newTask.description}
									onChange={(e) =>
										setNewTask({ ...newTask, description: e.target.value })
									}
									className="w-full px-3 py-2 border border-gray-300 dark:border-gray-700 bg-surface text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-theme-primary theme-transition"
									placeholder="输入任务描述..."
									rows={3}
								/>
							</div>

							<div>
								<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
									分类
								</label>
								<select
									value={newTask.category_id}
									onChange={(e) =>
										setNewTask({ ...newTask, category_id: e.target.value })
									}
									className="w-full px-3 py-2 border border-gray-300 dark:border-gray-700 bg-surface text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-theme-primary theme-transition"
								>
									<option value="">无分类</option>
									{categories.map((category) => (
										<option key={category.id} value={category.id}>
											{category.name}
										</option>
									))}
								</select>
							</div>

							<div>
								<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
									标签 (用逗号分隔)
								</label>
								<input
									type="text"
									value={newTask.tags}
									onChange={(e) =>
										setNewTask({ ...newTask, tags: e.target.value })
									}
									className="w-full px-3 py-2 border border-gray-300 dark:border-gray-700 bg-surface text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-theme-primary theme-transition"
									placeholder="工作, 项目, 重要..."
								/>
							</div>
						</div>

						<div className="flex justify-end space-x-3 mt-6">
							<button
								onClick={() => {
									setShowCreateDialog(false);
									setEditingTask(null);
									setNewTask({
										name: "",
										description: "",
										category_id: "",
										tags: "",
									});
								}}
								className="px-4 py-2 border border-gray-300 dark:border-gray-700 text-gray-700 dark:text-gray-300 bg-surface dark:bg-gray-700 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
							>
								取消
							</button>
							<button
								onClick={editingTask ? updateTask : createTask}
								className="px-4 py-2 bg-theme-primary text-white rounded-md bg-theme-primary-hover disabled:opacity-50 theme-transition"
								disabled={!newTask.name.trim()}
							>
								{editingTask ? "保存" : "创建"}
							</button>
						</div>
					</div>
				</div>
			)}
		</div>
	);
};

export default TaskManagement;
