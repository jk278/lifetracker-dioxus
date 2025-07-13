import { Database, Info, Settings } from "lucide-react";
import { memo, useRef, useEffect, useState } from "react";
import { useNavigation, getRouteDirection } from "../hooks/useRouter";
import type { RouteId } from "../types/router";
import About from "./About";
import { PageTransition } from "./Animation";
import {
	DataBackup,
	DataCleanup,
	DataExport,
	DataImport,
	DataManagement,
	DataSync,
} from "./DataManagement";
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

// 页面层级关系定义
const PAGE_HIERARCHY = {
	// 一级页面（系统概览的直接子页面）
	level1: ["data", "settings", "about"] as const,
	// 二级页面（数据管理的子页面）
	level2: ["data-export", "data-import", "data-backup", "data-sync", "data-cleanup"] as const,
} as const;

// 系统子页面路由列表
const SYSTEM_SUB_PAGES = [
	...PAGE_HIERARCHY.level1,
	...PAGE_HIERARCHY.level2,
] as const;

const SystemPage = memo(() => {
	const { currentRoute, currentSource, canGoBack, navigate, goBack } =
		useNavigation();

	// 窗口宽度检测
	const [isMobileLayout, setIsMobileLayout] = useState<boolean>(false);

	// 判断当前是否在系统页面的二级页面
	const isInSubPage =
		SYSTEM_SUB_PAGES.includes(
			currentRoute as (typeof SYSTEM_SUB_PAGES)[number],
		) && currentRoute !== "system";

	// 判断当前页面的层级
	const currentPageLevel = (() => {
		if (currentRoute === "system") return 0; // 系统概览
		if (PAGE_HIERARCHY.level1.includes(currentRoute as any)) return 1; // 一级页面
		if (PAGE_HIERARCHY.level2.includes(currentRoute as any)) return 2; // 二级页面
		return 0;
	})();

	// 记录前一个路由，用于比较层级差异
	const previousRouteRef = useRef<(RouteId | "system") | null>(null);

	// 计算前一个页面的层级
	const previousPageLevel = (() => {
		if (!previousRouteRef.current) return currentPageLevel;
		const prev = previousRouteRef.current;
		if (prev === "system") return 0;
		if (PAGE_HIERARCHY.level1.includes(prev as any)) return 1;
		if (PAGE_HIERARCHY.level2.includes(prev as any)) return 2;
		return 0;
	})();

	// 当从更深层级返回到较浅层级时，较浅层级页面使用 exitOnly
	const shouldUseExitOnly = currentPageLevel < previousPageLevel;

	// 当从较浅层级进入更深层级时，较浅层级页面使用 skipExitAnimation + exitDelay
	const shouldSkipExitAnimation = currentPageLevel > previousPageLevel;

	// 在路由变化后更新 previousRouteRef
	useEffect(() => {
		previousRouteRef.current = currentRoute;
	}, [currentRoute]);

	// 窗口宽度检测
	useEffect(() => {
		const handleResize = () => {
			const width = window.innerWidth;
			setIsMobileLayout(width < 768); // 768px 作为断点
		};

		// 初始化
		handleResize();

		// 监听窗口大小变化
		window.addEventListener("resize", handleResize);
		return () => window.removeEventListener("resize", handleResize);
	}, []);

	// 判断是否应该使用slide动画（桌面端的一级页面）
	const shouldUseSlideAnimation = !isMobileLayout && PAGE_HIERARCHY.level1.includes(currentRoute as any);

	// 计算动画方向
	const animationDirection = (() => {
		if (!previousRouteRef.current) return "forward";
		
		// 如果是桌面端的一级页面，使用路由方向检测
		if (shouldUseSlideAnimation) {
			return getRouteDirection(
				previousRouteRef.current as RouteId,
				currentRoute as RouteId,
				isMobileLayout
			);
		}
		
		// 其他情况使用层级判断
		return isInSubPage ? "forward" : "backward";
	})();

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

	// 渲染具体功能页面（去掉重复的顶部返回栏）
	const renderDetailView = () => {
		return (
			<div className="h-full flex flex-col overflow-hidden">
				{currentRoute === "data" && <DataManagement />}
				{currentRoute === "settings" && <SettingsComponent />}
				{currentRoute === "about" && <About />}
				{/* 数据管理子页面 */}
				{currentRoute === "data-export" && <DataExport />}
				{currentRoute === "data-import" && <DataImport />}
				{currentRoute === "data-backup" && <DataBackup />}
				{currentRoute === "data-sync" && <DataSync />}
				{currentRoute === "data-cleanup" && <DataCleanup />}
			</div>
		);
	};

	return (
		<div className="h-full bg-adaptive">
			<PageTransition
				routeKey={isInSubPage ? currentRoute : "system-overview"}
				animationCustom={{
					direction: isMobileLayout ? "horizontal" : "vertical",
					animationDirection: animationDirection,
					type: shouldUseSlideAnimation ? "slide" : "fade",
					// 当从更深层级返回到较浅层级时，较浅层级的页面使用 exitOnly
					// 这样从二级页面回到一级页面，或从一级页面回到系统概览都会有 exitOnly 效果
					exitOnly: shouldUseExitOnly,
					// 当从较浅层级进入更深层级时，较浅层级页面延迟后瞬间消失
					skipExitAnimation: shouldSkipExitAnimation,
					exitDelay: shouldSkipExitAnimation ? 0.1 : undefined,
				}}
			>
				{currentRoute === "system" || !isInSubPage
					? renderOverview()
					: renderDetailView()}
			</PageTransition>
		</div>
	);
});

export default SystemPage;
