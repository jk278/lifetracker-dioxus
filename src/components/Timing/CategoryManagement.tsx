import { invoke } from "@tauri-apps/api/core";
import { Edit, Folder, FolderOpen, Plus, Search, Trash2 } from "lucide-react";
import type React from "react";
import { useEffect, useState } from "react";
import type { Category } from "../../types";

interface CategoryManagementProps {
	onCategoriesUpdate: () => void;
}

// 图标渲染函数
const renderCategoryIcon = (icon: string | null | undefined, color: string) => {
	if (!icon) {
		return <FolderOpen className="h-5 w-5" style={{ color }} />;
	}

	// 如果是emoji（单个字符且不是英文字母），直接显示
	if (icon.length === 1 || icon.match(/[\u{1F300}-\u{1F9FF}]/u)) {
		return <span style={{ color, fontSize: "20px" }}>{icon}</span>;
	}

	// 如果是2个字符的emoji（如复合emoji），直接显示
	if (icon.length === 2 && icon.match(/[\u{1F300}-\u{1F9FF}]/u)) {
		return <span style={{ color, fontSize: "20px" }}>{icon}</span>;
	}

	// Material Design 图标名称映射
	const iconMap: { [key: string]: string } = {
		work: "💼",
		school: "📚",
		person: "👤",
		games: "🎮",
		fitness_center: "🏃",
		more_horiz: "📁",
		folder: "📁",
		business: "💼",
		study: "📚",
		learning: "📚",
		entertainment: "🎮",
		sports: "🏃",
		exercise: "🏃",
		personal: "👤",
		other: "📁",
	};

	const mappedIcon = iconMap[icon.toLowerCase()];
	if (mappedIcon) {
		return <span style={{ color, fontSize: "20px" }}>{mappedIcon}</span>;
	}

	// 如果都不匹配，尝试显示原始文本或默认图标
	if (icon.length < 10) {
		return <span style={{ color, fontSize: "16px" }}>{icon}</span>;
	}

	return <FolderOpen className="h-5 w-5" style={{ color }} />;
};

const CategoryManagement: React.FC<CategoryManagementProps> = ({
	onCategoriesUpdate,
}) => {
	const [categories, setCategories] = useState<Category[]>([]);
	const [searchTerm, setSearchTerm] = useState("");
	const [showCreateDialog, setShowCreateDialog] = useState(false);
	const [editingCategory, setEditingCategory] = useState<Category | null>(null);
	const [newCategory, setNewCategory] = useState({
		name: "",
		description: "",
		color: "#3B82F6",
		icon: "",
	});

	const predefinedColors = [
		"#3B82F6",
		"#EF4444",
		"#10B981",
		"#F59E0B",
		"#8B5CF6",
		"#EC4899",
		"#06B6D4",
		"#84CC16",
		"#F97316",
		"#6366F1",
		"#14B8A6",
		"#F43F5E",
	];

	const fetchCategories = async () => {
		try {
			const categoryList = await invoke<Category[]>("get_categories");
			setCategories(categoryList);
		} catch (error) {
			console.error("获取分类列表失败:", error);
		}
	};

	const createCategory = async () => {
		if (!newCategory.name.trim()) return;
		try {
			await invoke("create_category", {
				request: {
					name: newCategory.name,
					description: newCategory.description || null,
					color: newCategory.color,
					icon: newCategory.icon || null,
				},
			});

			setNewCategory({ name: "", description: "", color: "#3B82F6", icon: "" });
			setShowCreateDialog(false);
			fetchCategories();
			onCategoriesUpdate();
		} catch (error) {
			console.error("创建分类失败:", error);
		}
	};

	const updateCategory = async () => {
		if (!editingCategory || !newCategory.name.trim()) return;
		try {
			await invoke("update_category", {
				categoryId: editingCategory.id,
				request: {
					name: newCategory.name,
					description: newCategory.description || null,
					color: newCategory.color,
					icon: newCategory.icon || null,
				},
			});

			setEditingCategory(null);
			setNewCategory({ name: "", description: "", color: "#3B82F6", icon: "" });
			fetchCategories();
			onCategoriesUpdate();
		} catch (error) {
			console.error("更新分类失败:", error);
		}
	};

	const deleteCategory = async (categoryId: string) => {
		if (!confirm("确定要删除这个分类吗？删除后该分类下的任务将变为未分类。"))
			return;
		try {
			await invoke("delete_category", { categoryId });
			fetchCategories();
			onCategoriesUpdate();
		} catch (error) {
			console.error("删除分类失败:", error);
		}
	};

	const startEditCategory = (category: Category) => {
		setEditingCategory(category);
		setNewCategory({
			name: category.name,
			description: category.description || "",
			color: category.color,
			icon: category.icon || "",
		});
		setShowCreateDialog(true);
	};

	const filteredCategories = categories.filter(
		(category) =>
			category.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
			category.description?.toLowerCase().includes(searchTerm.toLowerCase()),
	);

	useEffect(() => {
		fetchCategories();
	}, []);

	return (
		<div className="space-y-6">
			<div className="flex items-center justify-between">
				<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
					分类管理
				</h3>
				<button
					onClick={() => {
						setEditingCategory(null);
						setNewCategory({
							name: "",
							description: "",
							color: "#3B82F6",
							icon: "",
						});
						setShowCreateDialog(true);
					}}
					className="flex items-center px-4 py-2 bg-theme-primary text-white rounded-lg bg-theme-primary-hover theme-transition"
				>
					<Plus className="h-4 w-4 mr-2" />
					新建分类
				</button>
			</div>

			<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-4">
				<div className="relative">
					<Search className="h-5 w-5 absolute left-3 top-3 text-gray-400 dark:text-gray-500" />
					<input
						type="text"
						value={searchTerm}
						onChange={(e) => setSearchTerm(e.target.value)}
						placeholder="搜索分类名称或描述..."
						className="pl-10 pr-4 py-2 w-full border border-gray-300 dark:border-gray-700 bg-surface text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 rounded-md focus:outline-none focus:ring-2 focus:ring-theme-primary theme-transition"
					/>
				</div>
			</div>

			{filteredCategories.length === 0 ? (
				<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20">
					<div className="text-center py-12">
						<Folder className="h-12 w-12 text-gray-400 dark:text-gray-500 mx-auto mb-4" />
						<h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
							暂无分类
						</h3>
						<p className="text-gray-500 dark:text-gray-400">
							{searchTerm
								? "没有符合条件的分类"
								: "创建您的第一个分类来组织任务"}
						</p>
					</div>
				</div>
			) : (
				<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
					{filteredCategories.map((category) => (
						<div
							key={category.id}
							className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-md dark:shadow-gray-700/20 hover:shadow-lg dark:hover:shadow-gray-700/30 transition-shadow"
						>
							<div className="p-6">
								<div className="flex items-center justify-between mb-4">
									<div className="flex items-center space-x-3">
										<div
											className="w-10 h-10 rounded-lg flex items-center justify-center"
											style={{ backgroundColor: category.color + "20" }}
										>
											{renderCategoryIcon(category.icon, category.color)}
										</div>
										<div>
											<h3 className="text-lg font-semibold text-gray-900 dark:text-white">
												{category.name}
											</h3>
											<p className="text-sm text-gray-500 dark:text-gray-400">
												{category.task_count || 0} 个任务
											</p>
										</div>
									</div>

									<div className="flex space-x-2">
										<button
											onClick={() => startEditCategory(category)}
											className="text-blue-600 dark:text-blue-400 hover:text-blue-900 dark:hover:text-blue-300 transition-colors"
										>
											<Edit className="h-4 w-4" />
										</button>
										<button
											onClick={() => deleteCategory(category.id)}
											className="text-red-600 dark:text-red-400 hover:text-red-900 dark:hover:text-red-300 transition-colors"
										>
											<Trash2 className="h-4 w-4" />
										</button>
									</div>
								</div>

								{category.description && (
									<p className="text-sm text-gray-600 dark:text-gray-300 mb-4">
										{category.description}
									</p>
								)}

								<div className="flex items-center justify-between">
									<div
										className="px-3 py-1 rounded-full text-xs font-medium"
										style={{
											backgroundColor: category.color + "20",
											color: category.color,
										}}
									>
										{category.name}
									</div>
									<div className="text-sm text-gray-500 dark:text-gray-400">
										创建于 {new Date(category.created_at).toLocaleDateString()}
									</div>
								</div>
							</div>
						</div>
					))}
				</div>
			)}

			{showCreateDialog && (
				<div className="fixed inset-0 bg-black/50 dark:bg-black/70 flex items-center justify-center z-50 !mt-0">
					<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 p-6 w-full max-w-md mx-4 shadow-xl">
						<h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
							{editingCategory ? "编辑分类" : "创建新分类"}
						</h3>

						<div className="space-y-4">
							<div>
								<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
									分类名称 *
								</label>
								<input
									type="text"
									value={newCategory.name}
									onChange={(e) =>
										setNewCategory({ ...newCategory, name: e.target.value })
									}
									className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
									placeholder="输入分类名称..."
									autoFocus
								/>
							</div>

							<div>
								<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
									分类描述
								</label>
								<textarea
									value={newCategory.description}
									onChange={(e) =>
										setNewCategory({
											...newCategory,
											description: e.target.value,
										})
									}
									className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
									placeholder="输入分类描述..."
									rows={3}
								/>
							</div>

							<div>
								<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
									分类颜色
								</label>
								<div className="flex flex-wrap gap-2">
									{predefinedColors.map((color) => (
										<button
											key={color}
											onClick={() => setNewCategory({ ...newCategory, color })}
											className={`w-8 h-8 rounded-full border-2 transition-colors ${
												newCategory.color === color
													? "border-gray-800 dark:border-gray-200"
													: "border-gray-300 dark:border-gray-600"
											}`}
											style={{ backgroundColor: color }}
										/>
									))}
								</div>
								<input
									type="color"
									value={newCategory.color}
									onChange={(e) =>
										setNewCategory({ ...newCategory, color: e.target.value })
									}
									className="mt-2 w-16 h-8 border border-gray-300 dark:border-gray-600 rounded cursor-pointer"
								/>
							</div>

							<div>
								<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
									图标 (可选)
								</label>
								<div className="flex flex-wrap gap-2 mb-3">
									{[
										"💼",
										"📚",
										"👤",
										"🎮",
										"🏃",
										"📁",
										"🎨",
										"💡",
										"🔧",
										"📊",
										"🛒",
										"🍔",
										"🏠",
										"🚗",
										"✈️",
										"🏥",
									].map((emoji) => (
										<button
											key={emoji}
											type="button"
											onClick={() =>
												setNewCategory({ ...newCategory, icon: emoji })
											}
											className={`w-10 h-10 rounded-lg border-2 flex items-center justify-center text-lg transition-colors ${
												newCategory.icon === emoji
													? "border-blue-500 bg-blue-50 dark:bg-blue-900"
													: "border-gray-300 dark:border-gray-600 hover:border-gray-400 dark:hover:border-gray-500"
											}`}
										>
											{emoji}
										</button>
									))}
								</div>
								<div className="flex gap-2">
									<input
										type="text"
										value={newCategory.icon}
										onChange={(e) =>
											setNewCategory({ ...newCategory, icon: e.target.value })
										}
										className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
										placeholder="📁 或者其他emoji..."
									/>
									<div className="w-10 h-10 border border-gray-300 dark:border-gray-600 rounded-md flex items-center justify-center bg-gray-50 dark:bg-gray-700">
										{renderCategoryIcon(newCategory.icon, newCategory.color)}
									</div>
								</div>
							</div>
						</div>

						<div className="flex justify-end space-x-3 mt-6">
							<button
								onClick={() => {
									setShowCreateDialog(false);
									setEditingCategory(null);
									setNewCategory({
										name: "",
										description: "",
										color: "#3B82F6",
										icon: "",
									});
								}}
								className="px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
							>
								取消
							</button>
							<button
								onClick={editingCategory ? updateCategory : createCategory}
								className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 transition-colors"
								disabled={!newCategory.name.trim()}
							>
								{editingCategory ? "保存" : "创建"}
							</button>
						</div>
					</div>
				</div>
			)}
		</div>
	);
};

export default CategoryManagement;
