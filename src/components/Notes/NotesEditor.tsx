import { memo, useState } from "react";
import { Edit, Save, X, Smile, Tag, Heart } from "lucide-react";

const NotesEditor = memo(() => {
	const [title, setTitle] = useState("");
	const [content, setContent] = useState("");
	const [mood, setMood] = useState<string>("");
	const [tags, setTags] = useState<string[]>([]);
	const [isFavorite, setIsFavorite] = useState(false);

	const moods = [
		{ value: "happy", label: "😊 开心", color: "text-yellow-500" },
		{ value: "sad", label: "😢 难过", color: "text-blue-500" },
		{ value: "neutral", label: "😐 平静", color: "text-gray-500" },
		{ value: "excited", label: "🤩 兴奋", color: "text-orange-500" },
		{ value: "stressed", label: "😰 压力", color: "text-red-500" },
		{ value: "relaxed", label: "😌 放松", color: "text-green-500" },
		{ value: "anxious", label: "😟 焦虑", color: "text-purple-500" },
		{ value: "confident", label: "😎 自信", color: "text-indigo-500" },
	];

	const handleSave = () => {
		// TODO: 实现保存功能
		console.log("保存笔记", { title, content, mood, tags, isFavorite });
	};

	const handleAddTag = (tag: string) => {
		if (tag && !tags.includes(tag)) {
			setTags([...tags, tag]);
		}
	};

	const handleRemoveTag = (tagToRemove: string) => {
		setTags(tags.filter(tag => tag !== tagToRemove));
	};

	return (
		<div className="space-y-6">
			{/* 顶部工具栏 */}
			<div className="flex items-center justify-between">
				<div className="flex items-center space-x-3">
					<Edit className="w-6 h-6 text-theme-primary" />
					<h1 className="text-xl font-bold text-gray-900 dark:text-white">
						笔记编辑器
					</h1>
				</div>
				<div className="flex items-center space-x-2">
					<button
						onClick={() => setIsFavorite(!isFavorite)}
						className={`p-2 rounded-lg transition-colors ${
							isFavorite
								? "text-red-500 bg-red-50 dark:bg-red-900/20"
								: "text-gray-600 dark:text-gray-300 hover:text-red-500 hover:bg-gray-100 dark:hover:bg-gray-700"
						}`}
					>
						<Heart className={`w-5 h-5 ${isFavorite ? "fill-current" : ""}`} />
					</button>
					<button
						onClick={handleSave}
						className="flex items-center space-x-2 px-3 py-2 bg-theme-primary text-white rounded-lg hover:bg-theme-primary-hover transition-colors"
					>
						<Save className="w-4 h-4" />
						<span className="text-sm font-medium">保存</span>
					</button>
				</div>
			</div>

			{/* 编辑区域 */}
			<div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
				<div className="p-6 space-y-4">
					{/* 标题输入 */}
					<div>
						<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
							标题
						</label>
						<input
							type="text"
							value={title}
							onChange={(e) => setTitle(e.target.value)}
							placeholder="请输入笔记标题..."
							className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-theme-primary focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400"
						/>
					</div>

					{/* 心情选择 */}
					<div>
						<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
							心情
						</label>
						<div className="flex flex-wrap gap-2">
							{moods.map((moodOption) => (
								<button
									key={moodOption.value}
									onClick={() => setMood(moodOption.value === mood ? "" : moodOption.value)}
									className={`px-3 py-1 rounded-full text-sm transition-colors ${
										mood === moodOption.value
											? "bg-theme-primary text-white"
											: "bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600"
									}`}
								>
									{moodOption.label}
								</button>
							))}
						</div>
					</div>

					{/* 标签输入 */}
					<div>
						<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
							标签
						</label>
						<div className="flex flex-wrap gap-2 mb-2">
							{tags.map((tag) => (
								<span
									key={tag}
									className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-theme-primary/10 text-theme-primary"
								>
									{tag}
									<button
										onClick={() => handleRemoveTag(tag)}
										className="ml-1 hover:text-theme-primary-hover"
									>
										<X className="w-3 h-3" />
									</button>
								</span>
							))}
						</div>
						<input
							type="text"
							placeholder="输入标签并按回车添加..."
							className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-theme-primary focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400"
							onKeyPress={(e) => {
								if (e.key === "Enter") {
									const tag = e.currentTarget.value.trim();
									if (tag) {
										handleAddTag(tag);
										e.currentTarget.value = "";
									}
								}
							}}
						/>
					</div>

					{/* 内容编辑 */}
					<div>
						<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
							内容
						</label>
						<textarea
							value={content}
							onChange={(e) => setContent(e.target.value)}
							placeholder="开始写下您的想法..."
							rows={12}
							className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-theme-primary focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 resize-none"
						/>
					</div>
				</div>
			</div>
		</div>
	);
});

export default NotesEditor; 