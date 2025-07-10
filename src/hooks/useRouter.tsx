import { createContext, useCallback, useContext, useState } from "react";
import type {
	NavigationSource,
	RouteId,
	RouteRecord,
	RouterConfig,
	RouterContext,
	RouteState,
} from "../types/router";

// 默认路由配置
const DEFAULT_CONFIG: RouterConfig = {
	rememberNavigation: true,
	defaultRoute: "timing",
};

// 存储键
const STORAGE_KEY = "lifetracker-router-state";
const CONFIG_KEY = "lifetracker-router-config";

// 路由上下文
const RouterContextInstance = createContext<RouterContext | null>(null);

// 路由顺序定义 - 用于方向检测
const ROUTE_ORDER = {
	mobile: ["timing", "accounting", "notes", "system"] as const,
	desktop: [
		"timing",
		"accounting",
		"notes",
		"data",
		"settings",
		"about",
	] as const,
};

// 标签页顺序定义
const TAB_ORDER = {
	accounting: ["overview", "accounts", "transactions", "stats"] as const,
	timing: ["dashboard", "tasks", "categories", "statistics"] as const,
	notes: ["overview", "editor", "library", "stats"] as const,
};

// 方向检测函数
export function getNavigationDirection(
	from: string,
	to: string,
	orderArray: readonly string[],
): "forward" | "backward" | "none" {
	const fromIndex = orderArray.indexOf(from);
	const toIndex = orderArray.indexOf(to);

	if (fromIndex === -1 || toIndex === -1) return "none";

	if (toIndex > fromIndex) return "forward";
	if (toIndex < fromIndex) return "backward";
	return "none";
}

// 标签页方向检测
export function getTabDirection(
	from: string,
	to: string,
	tabGroup: keyof typeof TAB_ORDER,
): "forward" | "backward" | "none" {
	const orderArray = TAB_ORDER[tabGroup];
	return getNavigationDirection(from, to, orderArray);
}

// 路由方向检测
export function getRouteDirection(
	from: RouteId,
	to: RouteId,
	isMobile: boolean,
): "forward" | "backward" | "none" {
	const orderArray = isMobile ? ROUTE_ORDER.mobile : ROUTE_ORDER.desktop;
	return getNavigationDirection(from, to, orderArray);
}

// 路由状态管理Hook
export function useRouterState(initialConfig?: Partial<RouterConfig>) {
	const [config, setConfig] = useState<RouterConfig>(() => {
		const savedConfig = localStorage.getItem(CONFIG_KEY);
		return {
			...DEFAULT_CONFIG,
			...initialConfig,
			...(savedConfig ? JSON.parse(savedConfig) : {}),
		};
	});

	const [state, setState] = useState<RouteState>(() => {
		// 如果开启记忆导航，尝试从localStorage恢复状态
		if (config.rememberNavigation) {
			const savedState = localStorage.getItem(STORAGE_KEY);
			if (savedState) {
				try {
					const parsed = JSON.parse(savedState);
					return {
						current: parsed.current || config.defaultRoute,
						source: parsed.source || "direct",
						stack: parsed.stack || [],
						canGoBack: parsed.stack?.length > 0,
					};
				} catch (error) {
					console.warn("Failed to restore router state:", error);
				}
			}
		}

		// 默认状态
		return {
			current: config.defaultRoute,
			source: "direct",
			stack: [],
			canGoBack: false,
		};
	});

	// 保存状态到localStorage
	const saveState = useCallback(
		(newState: RouteState) => {
			if (config.rememberNavigation) {
				localStorage.setItem(
					STORAGE_KEY,
					JSON.stringify({
						current: newState.current,
						source: newState.source,
						stack: newState.stack,
					}),
				);
			}
		},
		[config.rememberNavigation],
	);

	// 更新配置
	const updateConfig = useCallback(
		(newConfig: Partial<RouterConfig>) => {
			const updated = { ...config, ...newConfig };
			setConfig(updated);
			localStorage.setItem(CONFIG_KEY, JSON.stringify(updated));

			// 如果禁用了记忆导航，清除保存的状态
			if (!updated.rememberNavigation) {
				localStorage.removeItem(STORAGE_KEY);
			}
		},
		[config],
	);

	// 导航到指定路由
	const navigate = useCallback(
		(route: RouteId, source: NavigationSource = "direct") => {
			setState((prevState) => {
				const newRecord: RouteRecord = {
					route: prevState.current,
					source: prevState.source,
					timestamp: Date.now(),
				};

				// 如果是从系统页面导航，保留在栈中
				const newStack = [...prevState.stack];
				if (source === "system" || prevState.source === "system") {
					newStack.push(newRecord);
				} else {
					// 直接导航时清空栈
					newStack.length = 0;
				}

				const newState: RouteState = {
					current: route,
					source,
					stack: newStack,
					canGoBack: newStack.length > 0,
				};

				saveState(newState);
				return newState;
			});
		},
		[saveState],
	);

	// 返回上一级
	const goBack = useCallback(() => {
		setState((prevState) => {
			if (prevState.stack.length === 0) {
				return prevState;
			}

			const previousRecord = prevState.stack[prevState.stack.length - 1];
			const newStack = prevState.stack.slice(0, -1);

			const newState: RouteState = {
				current: previousRecord.route,
				source: previousRecord.source,
				stack: newStack,
				canGoBack: newStack.length > 0,
			};

			saveState(newState);
			return newState;
		});
	}, [saveState]);

	// 重置路由状态
	const reset = useCallback(() => {
		const newState: RouteState = {
			current: config.defaultRoute,
			source: "direct",
			stack: [],
			canGoBack: false,
		};
		setState(newState);
		saveState(newState);
	}, [config.defaultRoute, saveState]);

	const actions = {
		navigate,
		goBack,
		reset,
	};

	return {
		state,
		actions,
		config,
		updateConfig,
	};
}

// 路由提供者组件
export function RouterProvider({
	children,
	config,
}: {
	children: React.ReactNode;
	config?: Partial<RouterConfig>;
}) {
	const router = useRouterState(config);

	return (
		<RouterContextInstance.Provider value={router}>
			{children}
		</RouterContextInstance.Provider>
	);
}

// 使用路由Hook
export function useRouter() {
	const context = useContext(RouterContextInstance);
	if (!context) {
		throw new Error("useRouter must be used within a RouterProvider");
	}
	return context;
}

// 简化的导航Hook
export function useNavigation() {
	const { state, actions } = useRouter();
	return {
		currentRoute: state.current,
		currentSource: state.source,
		canGoBack: state.canGoBack,
		navigate: actions.navigate,
		goBack: actions.goBack,
	};
}
