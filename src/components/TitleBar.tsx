import { Clock, Minus, Square, X } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

interface TitleBarProps {
	title?: string;
}

const TitleBar: React.FC<TitleBarProps> = ({
	title = "TimeTracker - 时间追踪器",
}) => {
	const [isMaximized, setIsMaximized] = useState(false);

	// 检查窗口是否最大化
	const checkMaximized = useCallback(async () => {
		try {
			const { getCurrentWindow } = await import("@tauri-apps/api/window");
			const currentWindow = getCurrentWindow();
			const maximized = await currentWindow.isMaximized();
			setIsMaximized(maximized);
		} catch (error) {
			console.error("检查窗口状态失败:", error);
		}
	}, []);

	// 监听窗口状态变化
	useEffect(() => {
		checkMaximized();
	}, [checkMaximized]);

	// 最小化窗口
	const minimizeWindow = useCallback(async (e: React.MouseEvent) => {
		e.preventDefault();
		e.stopPropagation();

		try {
			const { getCurrentWindow } = await import("@tauri-apps/api/window");
			const currentWindow = getCurrentWindow();
			await currentWindow.minimize();
			console.log("窗口已最小化");
		} catch (error) {
			console.error("最小化窗口失败:", error);
		}
	}, []);

	// 最大化/还原窗口
	const toggleMaximize = useCallback(
		async (e: React.MouseEvent) => {
			e.preventDefault();
			e.stopPropagation();

			try {
				const { getCurrentWindow } = await import("@tauri-apps/api/window");
				const currentWindow = getCurrentWindow();
				await currentWindow.toggleMaximize();
				// 更新状态
				setTimeout(checkMaximized, 100);
				console.log("窗口状态已切换");
			} catch (error) {
				console.error("切换窗口状态失败:", error);
			}
		},
		[checkMaximized],
	);

	// 关闭窗口
	const closeWindow = useCallback(async (e: React.MouseEvent) => {
		e.preventDefault();
		e.stopPropagation();

		try {
			const { getCurrentWindow } = await import("@tauri-apps/api/window");
			const currentWindow = getCurrentWindow();
			await currentWindow.close();
			console.log("窗口已关闭");
		} catch (error) {
			console.error("关闭窗口失败:", error);
		}
	}, []);

	return (
		<div className="flex items-center justify-between h-8 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 select-none">
			{/* 左侧：应用图标和标题 - 可拖拽区域 */}
			<div
				data-tauri-drag-region="true"
				className="flex items-center space-x-2 pl-3 flex-1 h-full"
			>
				<Clock className="h-4 w-4 text-blue-600 dark:text-blue-400" />
				<span className="text-sm font-medium text-gray-700 dark:text-gray-200 truncate">
					{title}
				</span>
			</div>

			{/* 右侧：窗口控制按钮 - 非拖拽区域 */}
			<div className="flex items-center h-full">
				{/* 最小化按钮 */}
				<button
					type="button"
					onClick={minimizeWindow}
					className="h-full w-12 flex items-center justify-center hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
					title="最小化"
				>
					<Minus className="h-4 w-4 text-gray-600 dark:text-gray-300" />
				</button>

				{/* 最大化/还原按钮 */}
				<button
					type="button"
					onClick={toggleMaximize}
					className="h-full w-12 flex items-center justify-center hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
					title={isMaximized ? "还原" : "最大化"}
				>
					<Square className="h-3.5 w-3.5 text-gray-600 dark:text-gray-300" />
				</button>

				{/* 关闭按钮 */}
				<button
					type="button"
					onClick={closeWindow}
					className="h-full w-12 flex items-center justify-center hover:bg-red-500 hover:text-white transition-colors"
					title="关闭"
				>
					<X className="h-4 w-4 text-gray-600 dark:text-gray-300" />
				</button>
			</div>
		</div>
	);
};

export default TitleBar;
