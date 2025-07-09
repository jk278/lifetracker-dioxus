import { ArrowLeft, Database, Info, Settings } from "lucide-react";
import { memo } from "react";
import { useNavigation } from "../hooks/useRouter";
import type { RouteId } from "../types/router";
import About from "./About";
import { DataManagement } from "./DataManagement";
import SettingsComponent from "./Settings";

// 系统页面的子选项配置
const SYSTEM_ITEMS = [
	{
		id: "data",
		name: "数据管理",
		icon: Database,
		description: "导入导出、备份恢复",
	},
	{
		id: "settings",
		name: "应用设置",
		icon: Settings,
		description: "主题、偏好设置",
	},
	{
		id: "about",
		name: "关于应用",
		icon: Info,
		description: "版本信息、许可证",
	},
] as const;

const SystemPage = memo(() => {
	const { currentRoute, currentSource, canGoBack, navigate, goBack } =
		useNavigation();

	// 判断当前是否在系统页面的二级页面
	const isInSubPage = currentRoute !== "system" && currentSource === "system";

	// 返回到系统页面概览或上一级
	const handleBackToOverview = () => {
		if (canGoBack) {
			goBack();
		} else {
			// 如果无法返回，直接导航到系统页面
			navigate("system", "direct");
		}
	};

	// 导航到系统子页面
	const handleNavigateToSubPage = (subPageId: RouteId) => {
		navigate(subPageId, "system");
	};

	// 渲染系统页面概览
	const renderOverview = () => (
		<div className="h-full p-6 overflow-y-auto">
			<div className="max-w-4xl mx-auto">
				{/* 页面标题 */}
				<div className="mb-8">
					<h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
						系统管理
					</h1>
					<p className="text-gray-600 dark:text-gray-300">
						管理应用数据、设置和查看相关信息
					</p>
				</div>

				{/* 选项卡网格 */}
				<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
					{SYSTEM_ITEMS.map(({ id, name, icon: Icon, description }) => (
						<button
							key={id}
							onClick={() => handleNavigateToSubPage(id as RouteId)}
							className="p-6 surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 hover:border-theme-primary dark:hover:border-theme-primary transition-all duration-200 text-left group"
						>
							<div className="flex items-center mb-3">
								<div className="w-10 h-10 bg-theme-primary/10 rounded-lg flex items-center justify-center group-hover:bg-theme-primary/20 transition-colors">
									<Icon className="w-5 h-5 text-theme-primary" />
								</div>
							</div>
							<h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
								{name}
							</h3>
							<p className="text-sm text-gray-600 dark:text-gray-300">
								{description}
							</p>
						</button>
					))}
				</div>
			</div>
		</div>
	);

	// 渲染具体功能页面（带返回按钮）
	const renderDetailView = () => {
		const currentItem = SYSTEM_ITEMS.find((item) => item.id === currentRoute);

		return (
			<div className="h-full flex flex-col">
				{/* 返回导航栏 */}
				<div className="flex-shrink-0 p-4 border-b border-gray-200 dark:border-gray-700 surface-adaptive">
					<div className="flex items-center space-x-3">
						<button
							onClick={handleBackToOverview}
							className="flex items-center space-x-2 px-3 py-2 text-sm font-medium text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-50 dark:hover:bg-gray-700 rounded-lg transition-colors"
						>
							<ArrowLeft className="w-4 h-4" />
							<span>返回系统管理</span>
						</button>
						{currentItem && (
							<div className="flex items-center space-x-2 text-sm text-gray-500 dark:text-gray-400">
								<span>/</span>
								<span>{currentItem.name}</span>
							</div>
						)}
					</div>
				</div>

				{/* 具体页面内容 */}
				<div className="flex-1 overflow-hidden">
					{currentRoute === "data" && <DataManagement />}
					{currentRoute === "settings" && <SettingsComponent />}
					{currentRoute === "about" && <About />}
				</div>
			</div>
		);
	};

	return (
		<div className="h-full bg-adaptive">
			{/* 判断显示内容：系统页面概览 vs 二级页面 */}
			{currentRoute === "system"
				? renderOverview()
				: isInSubPage
					? renderDetailView()
					: renderOverview()}
		</div>
	);
});

export default SystemPage;
