import { Minus, Square, X } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

interface TitleBarProps {
	title?: string;
}

const TitleBar: React.FC<TitleBarProps> = ({ title = "LifeTracker" }) => {
	const [isMaximized, setIsMaximized] = useState(false);

	// 检测是否为移动端环境（仅根据设备类型判断，不考虑窗口宽度）
	const isMobile =
		typeof window !== "undefined" &&
		/Android|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
			navigator.userAgent,
		);

	// 检查窗口是否最大化
	const checkMaximized = useCallback(async () => {
		// 移动端跳过 API 调用
		if (isMobile) return;

		try {
			const { getCurrentWindow } = await import("@tauri-apps/api/window");
			const currentWindow = getCurrentWindow();
			const maximized = await currentWindow.isMaximized();
			setIsMaximized(maximized);
		} catch (error) {
			console.error("检查窗口状态失败:", error);
		}
	}, [isMobile]);

	// 监听窗口状态变化
	useEffect(() => {
		if (!isMobile) {
			checkMaximized();
		}
	}, [checkMaximized, isMobile]);

	// 最小化窗口
	const minimizeWindow = useCallback(
		async (e: React.MouseEvent) => {
			e.preventDefault();
			e.stopPropagation();

			// 移动端跳过 API 调用
			if (isMobile) return;

			try {
				const { getCurrentWindow } = await import("@tauri-apps/api/window");
				const currentWindow = getCurrentWindow();
				await currentWindow.minimize();
				console.log("窗口已最小化");
			} catch (error) {
				console.error("最小化窗口失败:", error);
			}
		},
		[isMobile],
	);

	// 最大化/还原窗口
	const toggleMaximize = useCallback(
		async (e: React.MouseEvent) => {
			e.preventDefault();
			e.stopPropagation();

			// 移动端跳过 API 调用
			if (isMobile) return;

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
		[checkMaximized, isMobile],
	);

	// 关闭窗口（现在改为最小化到托盘，不直接退出应用）
	const closeWindow = useCallback(
		async (e: React.MouseEvent) => {
			e.preventDefault();
			e.stopPropagation();

			// 移动端跳过 API 调用
			if (isMobile) return;

			try {
				const { getCurrentWindow } = await import("@tauri-apps/api/window");
				const currentWindow = getCurrentWindow();
				// 隐藏窗口到托盘
				await currentWindow.hide();
				console.log("窗口已隐藏到托盘");
			} catch (error) {
				console.error("隐藏到托盘失败:", error);
			}
		},
		[isMobile],
	);

	// 移动端直接不渲染
	if (isMobile) {
		return null;
	}

	return (
		<div className="flex items-center justify-between h-8 surface-adaptive border-b border-gray-200 dark:border-gray-700 select-none">
			{/* 左侧：应用标题 - 可拖拽区域 */}
			<div
				data-tauri-drag-region="true"
				className="flex items-center pl-3 flex-1 h-full"
			>
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
