import { type MotionProps, motion } from "framer-motion";
import { useMemo } from "react";

interface InteractiveButtonProps extends Omit<MotionProps, "children"> {
	children: React.ReactNode;
	onClick?: () => void;
	variant?: "primary" | "secondary" | "ghost" | "danger";
	size?: "sm" | "md" | "lg";
	disabled?: boolean;
	className?: string;
	title?: string;
}

const InteractiveButton: React.FC<InteractiveButtonProps> = ({
	children,
	onClick,
	variant = "primary",
	size = "md",
	disabled = false,
	className = "",
	title,
	...motionProps
}) => {
	// 使用useMemo优化性能
	const isMobile = useMemo(() => window.innerWidth < 768, []);

	// 样式类名 - 使用useMemo优化
	const variantClasses = useMemo(() => {
		switch (variant) {
			case "primary":
				return disabled
					? "bg-gray-300 text-gray-500 cursor-not-allowed"
					: "bg-blue-500 text-white hover:bg-blue-600 active:bg-blue-700";
			case "secondary":
				return disabled
					? "bg-gray-200 dark:bg-gray-700 text-gray-400 cursor-not-allowed"
					: "bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600 active:bg-gray-400 dark:active:bg-gray-500";
			case "ghost":
				return disabled
					? "text-gray-400 cursor-not-allowed"
					: "hover:bg-gray-100 dark:hover:bg-gray-700 active:bg-gray-200 dark:active:bg-gray-600";
			case "danger":
				return disabled
					? "bg-gray-300 text-gray-500 cursor-not-allowed"
					: "bg-red-500 text-white hover:bg-red-600 active:bg-red-700";
			default:
				return "";
		}
	}, [variant, disabled]);

	const sizeClasses = useMemo(() => {
		switch (size) {
			case "sm":
				return "px-3 py-1.5 text-sm";
			case "md":
				return "px-4 py-2 text-sm";
			case "lg":
				return "px-6 py-3 text-base";
			default:
				return "px-4 py-2 text-sm";
		}
	}, [size]);

	const buttonClass = useMemo(
		() => `
		font-medium rounded-lg 
		transition-colors duration-200 
		focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500
		${variantClasses}
		${sizeClasses}
		${className}
	`,
		[variantClasses, sizeClasses, className],
	);

	return (
		<motion.button
			onClick={disabled ? undefined : onClick}
			whileHover={disabled ? {} : { scale: isMobile ? 1.01 : 1.02 }}
			whileTap={disabled ? {} : { scale: isMobile ? 0.97 : 0.98 }}
			transition={{ duration: isMobile ? 0.1 : 0.15 }}
			title={title}
			className={buttonClass}
			style={{
				// 优化渲染性能
				willChange: "transform",
				backfaceVisibility: "hidden",
				WebkitBackfaceVisibility: "hidden",
				// 改善移动端性能
				WebkitTransform: "translateZ(0)",
				transform: "translateZ(0)",
				// 避免layout shift
				contain: "layout style paint",
			}}
			{...motionProps}
		>
			{children}
		</motion.button>
	);
};

export default InteractiveButton;
