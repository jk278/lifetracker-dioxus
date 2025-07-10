import { memo } from "react";
import { BarChart3, TrendingUp, Heart, Archive, Tag, Smile } from "lucide-react";

const NotesStats = memo(() => {
	// 模拟统计数据
	const stats = {
		total_notes: 24,
		favorite_notes: 8,
		archived_notes: 3,
		notes_this_week: 5,
		notes_this_month: 15,
		most_used_tags: [
			{ tag: "工作", count: 12, percentage: 50 },
			{ tag: "学习", count: 8, percentage: 33 },
			{ tag: "生活", count: 6, percentage: 25 },
			{ tag: "想法", count: 4, percentage: 17 },
			{ tag: "总结", count: 3, percentage: 13 },
		],
		mood_distribution: [
			{ mood: "happy", count: 10, percentage: 42 },
			{ mood: "excited", count: 6, percentage: 25 },
			{ mood: "neutral", count: 4, percentage: 17 },
			{ mood: "relaxed", count: 3, percentage: 13 },
			{ mood: "confident", count: 1, percentage: 4 },
		],
		daily_notes_trend: [
			{ date: "01-10", count: 2 },
			{ date: "01-11", count: 1 },
			{ date: "01-12", count: 3 },
			{ date: "01-13", count: 0 },
			{ date: "01-14", count: 2 },
			{ date: "01-15", count: 4 },
			{ date: "01-16", count: 1 },
		],
	};

	const getMoodInfo = (mood: string) => {
		const moodMap: { [key: string]: { emoji: string; label: string; color: string } } = {
			happy: { emoji: "😊", label: "开心", color: "bg-yellow-500" },
			sad: { emoji: "😢", label: "难过", color: "bg-blue-500" },
			neutral: { emoji: "😐", label: "平静", color: "bg-gray-500" },
			excited: { emoji: "🤩", label: "兴奋", color: "bg-orange-500" },
			stressed: { emoji: "😰", label: "压力", color: "bg-red-500" },
			relaxed: { emoji: "😌", label: "放松", color: "bg-green-500" },
			anxious: { emoji: "😟", label: "焦虑", color: "bg-purple-500" },
			confident: { emoji: "😎", label: "自信", color: "bg-indigo-500" },
		};
		return moodMap[mood] || { emoji: "😐", label: mood, color: "bg-gray-500" };
	};

	return (
		<div className="space-y-6">
			{/* 顶部工具栏 */}
			<div className="flex items-center justify-between">
				<div className="flex items-center space-x-3">
					<BarChart3 className="w-6 h-6 text-theme-primary" />
					<h1 className="text-xl font-bold text-gray-900 dark:text-white">
						笔记统计
					</h1>
				</div>
			</div>

			{/* 概览卡片 */}
			<div className="grid grid-cols-2 md:grid-cols-4 gap-4">
				<div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
					<div className="flex items-center justify-between">
						<div>
							<p className="text-sm text-gray-600 dark:text-gray-400">总笔记</p>
							<p className="text-2xl font-bold text-gray-900 dark:text-white">
								{stats.total_notes}
							</p>
						</div>
						<div className="w-8 h-8 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center">
							<BarChart3 className="w-4 h-4 text-blue-600 dark:text-blue-400" />
						</div>
					</div>
				</div>

				<div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
					<div className="flex items-center justify-between">
						<div>
							<p className="text-sm text-gray-600 dark:text-gray-400">收藏</p>
							<p className="text-2xl font-bold text-gray-900 dark:text-white">
								{stats.favorite_notes}
							</p>
						</div>
						<div className="w-8 h-8 bg-red-100 dark:bg-red-900/30 rounded-lg flex items-center justify-center">
							<Heart className="w-4 h-4 text-red-600 dark:text-red-400" />
						</div>
					</div>
				</div>

				<div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
					<div className="flex items-center justify-between">
						<div>
							<p className="text-sm text-gray-600 dark:text-gray-400">本周</p>
							<p className="text-2xl font-bold text-gray-900 dark:text-white">
								{stats.notes_this_week}
							</p>
						</div>
						<div className="w-8 h-8 bg-green-100 dark:bg-green-900/30 rounded-lg flex items-center justify-center">
							<TrendingUp className="w-4 h-4 text-green-600 dark:text-green-400" />
						</div>
					</div>
				</div>

				<div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
					<div className="flex items-center justify-between">
						<div>
							<p className="text-sm text-gray-600 dark:text-gray-400">归档</p>
							<p className="text-2xl font-bold text-gray-900 dark:text-white">
								{stats.archived_notes}
							</p>
						</div>
						<div className="w-8 h-8 bg-gray-100 dark:bg-gray-700 rounded-lg flex items-center justify-center">
							<Archive className="w-4 h-4 text-gray-600 dark:text-gray-400" />
						</div>
					</div>
				</div>
			</div>

			{/* 标签统计和心情分布 */}
			<div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
				{/* 常用标签 */}
				<div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
					<div className="p-4 border-b border-gray-200 dark:border-gray-700">
						<div className="flex items-center space-x-2">
							<Tag className="w-5 h-5 text-theme-primary" />
							<h3 className="font-semibold text-gray-900 dark:text-white">
								常用标签
							</h3>
						</div>
					</div>
					<div className="p-4 space-y-3">
						{stats.most_used_tags.map((tag, index) => (
							<div key={tag.tag} className="flex items-center justify-between">
								<div className="flex items-center space-x-3">
									<span className="w-6 h-6 bg-theme-primary/10 text-theme-primary rounded-full flex items-center justify-center text-xs font-medium">
										{index + 1}
									</span>
									<span className="text-gray-900 dark:text-white font-medium">
										{tag.tag}
									</span>
								</div>
								<div className="flex items-center space-x-2">
									<span className="text-sm text-gray-600 dark:text-gray-400">
										{tag.count}次
									</span>
									<div className="w-16 bg-gray-200 dark:bg-gray-700 rounded-full h-2">
										<div
											className="bg-theme-primary h-2 rounded-full transition-all duration-300"
											style={{ width: `${tag.percentage}%` }}
										/>
									</div>
								</div>
							</div>
						))}
					</div>
				</div>

				{/* 心情分布 */}
				<div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
					<div className="p-4 border-b border-gray-200 dark:border-gray-700">
						<div className="flex items-center space-x-2">
							<Smile className="w-5 h-5 text-theme-primary" />
							<h3 className="font-semibold text-gray-900 dark:text-white">
								心情分布
							</h3>
						</div>
					</div>
					<div className="p-4 space-y-3">
						{stats.mood_distribution.map((mood) => {
							const moodInfo = getMoodInfo(mood.mood);
							return (
								<div key={mood.mood} className="flex items-center justify-between">
									<div className="flex items-center space-x-3">
										<span className="text-lg">{moodInfo.emoji}</span>
										<span className="text-gray-900 dark:text-white font-medium">
											{moodInfo.label}
										</span>
									</div>
									<div className="flex items-center space-x-2">
										<span className="text-sm text-gray-600 dark:text-gray-400">
											{mood.count}次
										</span>
										<div className="w-16 bg-gray-200 dark:bg-gray-700 rounded-full h-2">
											<div
												className={`${moodInfo.color} h-2 rounded-full transition-all duration-300`}
												style={{ width: `${mood.percentage}%` }}
											/>
										</div>
									</div>
								</div>
							);
						})}
					</div>
				</div>
			</div>

			{/* 每日趋势 */}
			<div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
				<div className="p-4 border-b border-gray-200 dark:border-gray-700">
					<div className="flex items-center space-x-2">
						<TrendingUp className="w-5 h-5 text-theme-primary" />
						<h3 className="font-semibold text-gray-900 dark:text-white">
							最近7天趋势
						</h3>
					</div>
				</div>
				<div className="p-4">
					<div className="flex items-end justify-between space-x-2 h-32">
						{stats.daily_notes_trend.map((day, index) => {
							const maxCount = Math.max(...stats.daily_notes_trend.map(d => d.count));
							const height = maxCount > 0 ? (day.count / maxCount) * 100 : 0;
							
							return (
								<div key={index} className="flex flex-col items-center flex-1">
									<div
										className="w-full bg-theme-primary rounded-t-sm transition-all duration-300 hover:bg-theme-primary-hover"
										style={{ height: `${height}%`, minHeight: day.count > 0 ? "4px" : "0" }}
									/>
									<div className="mt-2 text-xs text-gray-600 dark:text-gray-400 text-center">
										{day.date}
									</div>
									<div className="text-xs text-gray-900 dark:text-white font-medium">
										{day.count}
									</div>
								</div>
							);
						})}
					</div>
				</div>
			</div>
		</div>
	);
});

export default NotesStats; 