import { BookOpen, Plus, Search } from "lucide-react";

function NotesPage() {
	return (
		<div className="h-full bg-adaptive flex flex-col">
			{/* 顶部工具栏 */}
			<div className="flex-shrink-0 p-4 surface-adaptive border-b border-gray-200 dark:border-gray-700">
				<div className="flex items-center justify-between">
					<div className="flex items-center space-x-3">
						<BookOpen className="w-6 h-6 text-theme-primary" />
						<h1 className="text-xl font-bold text-gray-900 dark:text-white">
							笔记
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
			</div>

			{/* 主内容区域 */}
			<div className="flex-1 flex items-center justify-center">
				<div className="text-center">
					<div className="w-24 h-24 bg-theme-primary/10 rounded-full flex items-center justify-center mx-auto mb-6">
						<BookOpen className="w-12 h-12 text-theme-primary" />
					</div>
					<h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
						笔记功能开发中
					</h2>
					<p className="text-gray-600 dark:text-gray-300 mb-8 max-w-sm">
						即将支持富文本编辑、心情追踪、标签分类等功能，打造完整的记录体验。
					</p>
					<div className="text-sm text-gray-500 dark:text-gray-400">
						敬请期待 🚀
					</div>
				</div>
			</div>
		</div>
	);
}

export default NotesPage; 