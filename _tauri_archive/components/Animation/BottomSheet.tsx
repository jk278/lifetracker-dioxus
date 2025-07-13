import { AnimatePresence, motion } from "framer-motion";
import { useEffect } from "react";

interface BottomSheetProps {
	isOpen: boolean;
	onClose: () => void;
	children: React.ReactNode;
	title?: string;
	height?: "auto" | "half" | "full";
	className?: string;
}

const BottomSheet: React.FC<BottomSheetProps> = ({
	isOpen,
	onClose,
	children,
	title,
	height = "auto",
	className = "",
}) => {
	// 防止背景滚动
	useEffect(() => {
		if (isOpen) {
			document.body.style.overflow = "hidden";
		} else {
			document.body.style.overflow = "unset";
		}

		return () => {
			document.body.style.overflow = "unset";
		};
	}, [isOpen]);

	const getHeightClass = () => {
		switch (height) {
			case "half":
				return "h-1/2";
			case "full":
				return "h-full";
			case "auto":
			default:
				return "max-h-[80vh]";
		}
	};

	return (
		<AnimatePresence>
			{isOpen && (
				<>
					{/* 背景遮罩 */}
					<motion.div
						initial={{ opacity: 0 }}
						animate={{ opacity: 1 }}
						exit={{ opacity: 0 }}
						onClick={onClose}
						className="fixed inset-0 bg-black bg-opacity-50 z-40"
						style={{ backdropFilter: "blur(4px)" }}
					/>

					{/* 底部弹出层 */}
					<motion.div
						initial={{ y: "100%" }}
						animate={{ y: 0 }}
						exit={{ y: "100%" }}
						transition={{
							type: "spring" as const,
							damping: 25,
							stiffness: 400,
							duration: 0.3,
						}}
						className={`
              fixed bottom-0 left-0 right-0 
              bg-white dark:bg-gray-800 
              rounded-t-xl shadow-xl z-50
              ${getHeightClass()}
              ${className}
            `}
						style={{
							willChange: "transform",
						}}
					>
						{/* 拖拽指示器 */}
						<div className="w-full flex justify-center pt-2 pb-1">
							<div className="w-10 h-1 bg-gray-300 dark:bg-gray-600 rounded-full" />
						</div>

						{/* 标题栏 */}
						{title && (
							<div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
								<h2 className="text-lg font-semibold text-gray-900 dark:text-white">
									{title}
								</h2>
								<button
									onClick={onClose}
									className="p-2 rounded-full hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
								>
									<svg
										className="w-5 h-5"
										fill="none"
										stroke="currentColor"
										viewBox="0 0 24 24"
									>
										<path
											strokeLinecap="round"
											strokeLinejoin="round"
											strokeWidth={2}
											d="M6 18L18 6M6 6l12 12"
										/>
									</svg>
								</button>
							</div>
						)}

						{/* 内容区域 */}
						<div className="p-4 overflow-y-auto flex-1">{children}</div>
					</motion.div>
				</>
			)}
		</AnimatePresence>
	);
};

export default BottomSheet;
