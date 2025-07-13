import { memo, useState } from "react";
import { TabTransition } from "./Animation";
import NotesOverview from "./Notes/NotesOverview";
import NotesEditor from "./Notes/NotesEditor";
import NotesLibrary from "./Notes/NotesLibrary";
import NotesStats from "./Notes/NotesStats";

const NotesPage = memo(() => {
	const [activeTab, setActiveTab] = useState<
		"overview" | "editor" | "library" | "stats"
	>("overview");
	const [previousTab, setPreviousTab] = useState<
		"overview" | "editor" | "library" | "stats"
	>("overview");

	const tabs = [
		{ key: "overview", label: "概览" },
		{ key: "editor", label: "编辑器" },
		{ key: "library", label: "笔记库" },
		{ key: "stats", label: "统计" },
	];

	return (
		<div className="flex flex-col h-full">
			{/* 内部标签导航 - 固定在顶部 */}
			<div className="flex-shrink-0 surface-adaptive border-b border-gray-200 dark:border-gray-700 overflow-x-auto sticky top-0 z-10 pt-2 md:pt-4">
				<div className="flex px-0 md:px-6">
					{tabs.map((tab) => (
						<div key={tab.key} className="relative">
							<button
								onClick={() => {
									setPreviousTab(activeTab);
									setActiveTab(tab.key as any);
								}}
								className={`px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap ${
									activeTab === tab.key
										? "text-theme-primary"
										: "text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
								}`}
							>
								{tab.label}
							</button>

							{/* 现代化的选中指示器 - 底部细线 */}
							<div
								className={`absolute bottom-0 left-1/2 transform -translate-x-1/2 h-0.5 bg-theme-primary transition-all duration-300 ease-out ${
									activeTab === tab.key ? "w-8 opacity-100" : "w-0 opacity-0"
								}`}
							/>
						</div>
					))}
				</div>
			</div>

			{/* 对应内容 - 可滚动区域 */}
			<div className="flex-1 overflow-y-auto py-4 px-4 md:px-6 scroll-container">
				<TabTransition
					activeKey={activeTab}
					direction="right"
					previousTab={previousTab}
					tabGroup="notes"
				>
					{activeTab === "overview" && <NotesOverview />}
					{activeTab === "editor" && <NotesEditor />}
					{activeTab === "library" && <NotesLibrary />}
					{activeTab === "stats" && <NotesStats />}
				</TabTransition>
			</div>
		</div>
	);
});

export default NotesPage;
