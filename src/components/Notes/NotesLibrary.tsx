import { memo, useState } from "react";
import { Library, Search, Filter, Grid, List, Heart, Archive, Calendar } from "lucide-react";

const NotesLibrary = memo(() => {
	const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
	const [searchQuery, setSearchQuery] = useState("");
	const [selectedFilter, setSelectedFilter] = useState<"all" | "favorite" | "archived">("all");

	const filters = [
		{ value: "all", label: "全部", icon: Library },
		{ value: "favorite", label: "收藏", icon: Heart },
		{ value: "archived", label: "归档", icon: Archive },
	];

	// 模拟笔记数据
	const mockNotes = [
		{
			id: "1",
			title: "今日工作总结",
			content: "今天完成了项目的主要功能开发，遇到了一些技术难题但都已解决...",
			mood: "happy",
			tags: ["工作", "总结"],
			isFavorite: true,
			isArchived: false,
			createdAt: "2024-01-15T10:30:00Z",
			updatedAt: "2024-01-15T18:45:00Z",
		},
		{
			id: "2",
			title: "学习笔记 - React Hooks",
			content: "useCallback 和 useMemo 的区别与使用场景...",
			mood: "excited",
			tags: ["学习", "React", "前端"],
			isFavorite: false,
			isArchived: false,
			createdAt: "2024-01-14T14:20:00Z",
			updatedAt: "2024-01-14T16:30:00Z",
		},
	];

	const filteredNotes = mockNotes.filter(note => {
		if (selectedFilter === "favorite" && !note.isFavorite) return false;
		if (selectedFilter === "archived" && !note.isArchived) return false;
		if (searchQuery && !note.title.toLowerCase().includes(searchQuery.toLowerCase()) &&
			!note.content.toLowerCase().includes(searchQuery.toLowerCase())) return false;
		return true;
	});

	const getMoodEmoji = (mood: string) => {
		const moodMap: { [key: string]: string } = {
			happy: "😊",
			sad: "😢",
			neutral: "😐",
			excited: "🤩",
			stressed: "😰",
			relaxed: "😌",
			anxious: "😟",
			confident: "😎",
		};
		return moodMap[mood] || "";
	};

	const formatDate = (dateString: string) => {
		const date = new Date(dateString);
		return date.toLocaleDateString("zh-CN", {
			month: "short",
			day: "numeric",
			hour: "2-digit",
			minute: "2-digit",
		});
	};

	return (
		<div className="space-y-6">
			{/* 顶部工具栏 */}
			<div className="flex items-center justify-between">
				<div className="flex items-center space-x-3">
					<Library className="w-6 h-6 text-theme-primary" />
					<h1 className="text-xl font-bold text-gray-900 dark:text-white">
						笔记库
					</h1>
				</div>
				<div className="flex items-center space-x-2">
					<button
						onClick={() => setViewMode(viewMode === "grid" ? "list" : "grid")}
						className="p-2 text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
					>
						{viewMode === "grid" ? <List className="w-5 h-5" /> : <Grid className="w-5 h-5" />}
					</button>
				</div>
			</div>

			{/* 搜索和过滤 */}
			<div className="flex flex-col sm:flex-row gap-4">
				{/* 搜索框 */}
				<div className="relative flex-1">
					<Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" />
					<input
						type="text"
						value={searchQuery}
						onChange={(e) => setSearchQuery(e.target.value)}
						placeholder="搜索笔记..."
						className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-theme-primary focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
					/>
				</div>

				{/* 过滤器 */}
				<div className="flex space-x-2">
					{filters.map((filter) => (
						<button
							key={filter.value}
							onClick={() => setSelectedFilter(filter.value as any)}
							className={`flex items-center space-x-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
								selectedFilter === filter.value
									? "bg-theme-primary text-white"
									: "bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600"
							}`}
						>
							<filter.icon className="w-4 h-4" />
							<span>{filter.label}</span>
						</button>
					))}
				</div>
			</div>

			{/* 笔记列表 */}
			{filteredNotes.length === 0 ? (
				<div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-12">
					<div className="text-center">
						<Library className="w-12 h-12 text-gray-400 mx-auto mb-4" />
						<h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
							{searchQuery ? "未找到相关笔记" : "还没有笔记"}
						</h3>
						<p className="text-gray-600 dark:text-gray-400">
							{searchQuery ? "尝试调整搜索条件" : "开始创建您的第一篇笔记"}
						</p>
					</div>
				</div>
			) : (
				<div className={
					viewMode === "grid"
						? "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
						: "space-y-4"
				}>
					{filteredNotes.map((note) => (
						<div
							key={note.id}
							className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 hover:shadow-md transition-shadow cursor-pointer"
						>
							{/* 笔记头部 */}
							<div className="flex items-start justify-between mb-3">
								<h3 className="font-semibold text-gray-900 dark:text-white truncate flex-1">
									{note.title}
								</h3>
								<div className="flex items-center space-x-1 ml-2 flex-shrink-0">
									{note.mood && (
										<span className="text-lg">
											{getMoodEmoji(note.mood)}
										</span>
									)}
									{note.isFavorite && (
										<Heart className="w-4 h-4 text-red-500 fill-current" />
									)}
									{note.isArchived && (
										<Archive className="w-4 h-4 text-gray-500" />
									)}
								</div>
							</div>

							{/* 笔记内容预览 */}
							<p className="text-gray-600 dark:text-gray-300 text-sm mb-3 line-clamp-3">
								{note.content}
							</p>

							{/* 标签 */}
							{note.tags.length > 0 && (
								<div className="flex flex-wrap gap-1 mb-3">
									{note.tags.map((tag) => (
										<span
											key={tag}
											className="px-2 py-1 bg-theme-primary/10 text-theme-primary text-xs rounded-full"
										>
											{tag}
										</span>
									))}
								</div>
							)}

							{/* 时间信息 */}
							<div className="flex items-center text-xs text-gray-500 dark:text-gray-400">
								<Calendar className="w-3 h-3 mr-1" />
								<span>更新于 {formatDate(note.updatedAt)}</span>
							</div>
						</div>
					))}
				</div>
			)}
		</div>
	);
});

export default NotesLibrary; 