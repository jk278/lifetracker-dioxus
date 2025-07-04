import { Clock, Github, Globe, Heart, Mail } from "lucide-react";
import type React from "react";
import { useEffect, useState } from "react";

const About: React.FC = () => {
	const [showDetails, setShowDetails] = useState(false);
	const [showSystemInfo, setShowSystemInfo] = useState(false);
	const [showLicense, setShowLicense] = useState(false);
	const [appInfo, setAppInfo] = useState<any>(null);

	// 获取应用信息
	useEffect(() => {
		// 模拟应用信息，实际可以从 Tauri 命令获取
		const info = {
			name: "LifeTracker",
			version: "0.1.0",
			description: "综合性的生活追踪和管理工具",
			author: "LifeTracker Team",
			email: "contact@lifetracker.dev",
			website: "https://lifetracker.dev",
			repository: "https://github.com/lifetracker/lifetracker",
			license: "MIT",
			buildDate: "2024-01-15",
			buildTarget: "Windows x64",
		};
		setAppInfo(info);
	}, []);

	const features = [
		{
			icon: "⏱️",
			title: "精确的时间跟踪",
			desc: "记录每个任务的开始和结束时间",
		},
		{ icon: "📊", title: "详细的统计分析", desc: "提供多维度的时间使用分析" },
		{ icon: "🏷️", title: "灵活的分类管理", desc: "支持自定义分类和标签" },
		{ icon: "📈", title: "趋势分析", desc: "分析工作模式和效率趋势" },
		{ icon: "🔔", title: "智能提醒", desc: "休息提醒和目标达成通知" },
		{ icon: "💾", title: "数据备份", desc: "支持数据导出和备份恢复" },
		{ icon: "🎨", title: "主题定制", desc: "多种主题和界面定制选项" },
		{ icon: "⌨️", title: "快捷键支持", desc: "提高操作效率的快捷键" },
	];

	const acknowledgments = [
		{ name: "Rust", desc: "系统编程语言", url: "https://rust-lang.org" },
		{ name: "Tauri", desc: "跨平台应用框架", url: "https://tauri.app" },
		{ name: "React", desc: "用户界面库", url: "https://react.dev" },
		{
			name: "TypeScript",
			desc: "类型安全的JavaScript",
			url: "https://typescriptlang.org",
		},
		{
			name: "Tailwind CSS",
			desc: "实用优先的CSS框架",
			url: "https://tailwindcss.com",
		},
		{ name: "SQLite", desc: "嵌入式数据库", url: "https://sqlite.org" },
	];

	const systemInfo = {
		os: navigator.platform,
		userAgent: navigator.userAgent,
		language: navigator.language,
		memoryUsage: "约 80MB",
		screenResolution: `${screen.width}x${screen.height}`,
	};

	if (!appInfo) {
		return (
			<div className="flex items-center justify-center h-64">
				<div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" />
			</div>
		);
	}

	return (
		<div className="max-w-4xl mx-auto p-6 space-y-8">
			{/* 页面标题 */}
			<div className="text-center">
				<h2 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
					关于 LifeTracker
				</h2>
				<p className="text-gray-600 dark:text-gray-400">
					了解更多关于我们的应用程序
				</p>
			</div>

			{/* 应用图标和基本信息 */}
			<div className="text-center space-y-6">
				{/* 应用图标 */}
				<div className="flex justify-center">
					<div className="w-20 h-20 bg-theme-primary rounded-full flex items-center justify-center">
						<Clock className="h-10 w-10 text-white" />
					</div>
				</div>

				{/* 应用名称和版本 */}
				<div>
					<h1 className="text-4xl font-bold text-theme-primary mb-2">
						{appInfo.name}
					</h1>
					<p className="text-lg text-gray-500 dark:text-gray-400">
						版本 {appInfo.version}
					</p>
					<p className="text-gray-600 dark:text-gray-300 mt-2">
						{appInfo.description}
					</p>
				</div>
			</div>

			{/* 控制选项 */}
			<div className="flex justify-center space-x-6">
				<label className="flex items-center space-x-2">
					<input
						type="checkbox"
						checked={showDetails}
						onChange={(e) => setShowDetails(e.target.checked)}
						className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
					/>
					<span className="text-sm text-gray-600 dark:text-gray-400">
						显示详细信息
					</span>
				</label>
				<label className="flex items-center space-x-2">
					<input
						type="checkbox"
						checked={showSystemInfo}
						onChange={(e) => setShowSystemInfo(e.target.checked)}
						className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
					/>
					<span className="text-sm text-gray-600 dark:text-gray-400">
						显示系统信息
					</span>
				</label>
				<label className="flex items-center space-x-2">
					<input
						type="checkbox"
						checked={showLicense}
						onChange={(e) => setShowLicense(e.target.checked)}
						className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
					/>
					<span className="text-sm text-gray-600 dark:text-gray-400">
						显示许可证
					</span>
				</label>
			</div>

			{/* 基本信息 */}
			<div className="bg-surface rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
				<h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
					基本信息
				</h3>
				<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
					<div className="space-y-2">
						<div className="flex justify-between">
							<span className="text-gray-600 dark:text-gray-400">开发者:</span>
							<span className="text-gray-900 dark:text-white">
								{appInfo.author}
							</span>
						</div>
						<div className="flex justify-between">
							<span className="text-gray-600 dark:text-gray-400">许可证:</span>
							<span className="text-gray-900 dark:text-white">
								{appInfo.license}
							</span>
						</div>
						<div className="flex justify-between">
							<span className="text-gray-600 dark:text-gray-400">
								构建日期:
							</span>
							<span className="text-gray-900 dark:text-white">
								{appInfo.buildDate}
							</span>
						</div>
					</div>
					<div className="space-y-2">
						<div className="flex justify-between">
							<span className="text-gray-600 dark:text-gray-400">
								构建目标:
							</span>
							<span className="text-gray-900 dark:text-white">
								{appInfo.buildTarget}
							</span>
						</div>
						<div className="flex justify-between">
							<span className="text-gray-600 dark:text-gray-400">框架:</span>
							<span className="text-gray-900 dark:text-white">
								Tauri + React
							</span>
						</div>
						<div className="flex justify-between">
							<span className="text-gray-600 dark:text-gray-400">状态:</span>
							<span className="text-green-600 dark:text-green-400">运行中</span>
						</div>
					</div>
				</div>
			</div>

			{/* 相关链接 */}
			<div className="bg-surface rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
				<h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
					相关链接
				</h3>
				<div className="flex flex-wrap gap-4">
					<a
						href={appInfo.website}
						target="_blank"
						rel="noopener noreferrer"
						className="flex items-center space-x-2 px-4 py-2 bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 rounded-lg hover:bg-blue-100 dark:hover:bg-blue-900/30 transition-colors"
					>
						<Globe className="h-4 w-4" />
						<span>官方网站</span>
					</a>
					<a
						href={`mailto:${appInfo.email}`}
						className="flex items-center space-x-2 px-4 py-2 bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400 rounded-lg hover:bg-green-100 dark:hover:bg-green-900/30 transition-colors"
					>
						<Mail className="h-4 w-4" />
						<span>联系我们</span>
					</a>
					<a
						href={appInfo.repository}
						target="_blank"
						rel="noopener noreferrer"
						className="flex items-center space-x-2 px-4 py-2 bg-gray-50 dark:bg-gray-700 text-gray-600 dark:text-gray-400 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
					>
						<Github className="h-4 w-4" />
						<span>源代码</span>
					</a>
				</div>
			</div>

			{/* 主要功能 */}
			<div className="bg-surface rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
				<h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
					主要功能
				</h3>
				<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
					{features.map((feature) => (
						<div key={feature.title} className="flex items-start space-x-3">
							<span className="text-2xl">{feature.icon}</span>
							<div>
								<h4 className="font-medium text-gray-900 dark:text-white">
									{feature.title}
								</h4>
								<p className="text-sm text-gray-600 dark:text-gray-400">
									{feature.desc}
								</p>
							</div>
						</div>
					))}
				</div>
			</div>

			{/* 系统信息 */}
			{showSystemInfo && (
				<div className="bg-surface rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
					<h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
						系统信息
					</h3>
					<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
						<div className="space-y-2">
							<div className="flex items-center justify-between">
								<span className="text-gray-600 dark:text-gray-400">
									操作系统:
								</span>
								<span className="text-gray-900 dark:text-white">
									{systemInfo.os}
								</span>
							</div>
							<div className="flex items-center justify-between">
								<span className="text-gray-600 dark:text-gray-400">语言:</span>
								<span className="text-gray-900 dark:text-white">
									{systemInfo.language}
								</span>
							</div>
							<div className="flex items-center justify-between">
								<span className="text-gray-600 dark:text-gray-400">
									内存使用:
								</span>
								<span className="text-gray-900 dark:text-white">
									{systemInfo.memoryUsage}
								</span>
							</div>
						</div>
						<div className="space-y-2">
							<div className="flex items-center justify-between">
								<span className="text-gray-600 dark:text-gray-400">
									屏幕分辨率:
								</span>
								<span className="text-gray-900 dark:text-white">
									{systemInfo.screenResolution}
								</span>
							</div>
						</div>
					</div>
				</div>
			)}

			{/* 许可证信息 */}
			{showLicense && (
				<div className="bg-surface rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
					<h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
						许可证信息
					</h3>
					<div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
						<h4 className="font-medium text-gray-900 dark:text-white mb-2">
							MIT License
						</h4>
						<p className="text-sm text-gray-600 dark:text-gray-300 leading-relaxed">
							Copyright (c) 2024 LifeTracker Team
							<br />
							<br />
							Permission is hereby granted, free of charge, to any person
							obtaining a copy of this software and associated documentation
							files (the "Software"), to deal in the Software without
							restriction, including without limitation the rights to use, copy,
							modify, merge, publish, distribute, sublicense, and/or sell copies
							of the Software, and to permit persons to whom the Software is
							furnished to do so, subject to the following conditions:
							<br />
							<br />
							The above copyright notice and this permission notice shall be
							included in all copies or substantial portions of the Software.
							<br />
							<br />
							THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
							EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
							MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
							NONINFRINGEMENT.
						</p>
					</div>
				</div>
			)}

			{/* 版本历史 */}
			{showDetails && (
				<div className="bg-surface rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
					<h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
						版本历史
					</h3>
					<div className="space-y-3">
						{[
							{
								version: "v1.0.0",
								date: "2024-01-15",
								desc: "首个正式版本发布",
							},
							{
								version: "v0.9.0",
								date: "2024-01-01",
								desc: "添加统计分析功能",
							},
							{ version: "v0.8.0", date: "2023-12-15", desc: "实现Tauri界面" },
							{ version: "v0.7.0", date: "2023-12-01", desc: "添加数据库支持" },
							{
								version: "v0.6.0",
								date: "2023-11-15",
								desc: "实现核心时间跟踪功能",
							},
						].map((item) => (
							<div key={item.version} className="flex items-center space-x-4">
								<span className="font-medium text-blue-600 dark:text-blue-400 w-16">
									{item.version}
								</span>
								<span className="text-gray-500 dark:text-gray-400 w-20">
									{item.date}
								</span>
								<span className="text-gray-900 dark:text-white">
									{item.desc}
								</span>
							</div>
						))}
					</div>
				</div>
			)}

			{/* 致谢 */}
			{showDetails && (
				<div className="bg-surface rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
					<h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-4 flex items-center space-x-2">
						<Heart className="h-5 w-5 text-red-500" />
						<span>致谢</span>
					</h3>
					<p className="text-gray-600 dark:text-gray-400 mb-4">
						感谢以下开源项目和贡献者:
					</p>
					<div className="grid grid-cols-1 md:grid-cols-2 gap-3">
						{acknowledgments.map((item) => (
							<div key={item.name} className="flex items-center space-x-3">
								<span className="w-2 h-2 bg-blue-600 rounded-full" />
								<div>
									<span className="font-medium text-gray-900 dark:text-white">
										{item.name}
									</span>
									<span className="text-gray-600 dark:text-gray-400">
										{" "}
										- {item.desc}
									</span>
								</div>
							</div>
						))}
					</div>
				</div>
			)}

			{/* 底部版权信息 */}
			<div className="text-center py-6">
				<p className="text-sm text-gray-500 dark:text-gray-400">
					© 2024 LifeTracker Team. All rights reserved.
				</p>
			</div>
		</div>
	);
};

export default About;
