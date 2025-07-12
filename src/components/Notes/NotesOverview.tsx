import { memo } from "react";
import { BookOpen, Plus, Search, Heart, Archive, TrendingUp } from "lucide-react";

const NotesOverview = memo(() => {
	return (
		<div className="space-y-6">
			{/* 顶部工具栏 */}
			<div className="flex items-center justify-between">
				<div className="flex items-center space-x-3">
					<BookOpen className="w-6 h-6 text-theme-primary" />
					<h1 className="text-xl font-bold text-gray-900 dark:text-white">
						笔记概览
					</h1>
				</div>
				<div className="flex items-center space-x-2">
					<button className="p-2 text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors">
						<Search className="w-5 h-5" />
					</button>
					<button className="flex items-center space-x-2 px-3 py-2 bg-theme-primary text-white rounded-lg hover:bg-theme-primary-hover transition-colors">
						<Plus className="w-4 h-4" />
						<span className="text-sm font-medium">新建笔记</span>
					</button>
				</div>
			</div>

			{/* 统计卡片 */}
			<div className="grid grid-cols-1 md:grid-cols-3 gap-4">
				<div className="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
					<div className="flex items-center justify-between">
						<div>
							<p className="text-sm font-medium text-gray-600 dark:text-gray-400">
								总笔记
							</p>
							<p className="text-2xl font-bold text-gray-900 dark:text-white">
								0
							</p>
						</div>
						<BookOpen className="w-8 h-8 text-blue-500" />
					</div>
				</div>

				<div className="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
					<div className="flex items-center justify-between">
						<div>
							<p className="text-sm font-medium text-gray-600 dark:text-gray-400">
								收藏笔记
							</p>
							<p className="text-2xl font-bold text-gray-900 dark:text-white">
								0
							</p>
						</div>
						<Heart className="w-8 h-8 text-red-500" />
					</div>
				</div>

				<div className="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
					<div className="flex items-center justify-between">
						<div>
							<p className="text-sm font-medium text-gray-600 dark:text-gray-400">
								本周新增
							</p>
							<p className="text-2xl font-bold text-gray-900 dark:text-white">
								0
							</p>
						</div>
						<TrendingUp className="w-8 h-8 text-green-500" />
					</div>
				</div>
			</div>

			{/* 最近笔记 */}
			<div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
				<div className="p-6 border-b border-gray-200 dark:border-gray-700">
					<h2 className="text-lg font-semibold text-gray-900 dark:text-white">
						最近笔记
					</h2>
				</div>
				<div className="p-6">
					<div className="text-center py-8">
						<BookOpen className="w-12 h-12 text-gray-400 mx-auto mb-4" />
						<h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
							还没有笔记
						</h3>
						<p className="text-gray-600 dark:text-gray-400 mb-4">
							开始记录您的想法和灵感吧
						</p>
						<button className="flex items-center space-x-2 px-4 py-2 bg-theme-primary text-white rounded-lg hover:bg-theme-primary-hover transition-colors mx-auto">
							<Plus className="w-4 h-4" />
							<span>创建第一篇笔记</span>
						</button>
					</div>
				</div>
			</div>
		</div>
	);
});

export default NotesOverview; 