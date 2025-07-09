import { type MotionProps, motion } from "framer-motion";

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
	// 检测移动端并调整动画参数
	const isMobile = window.innerWidth < 768;

	// 动画变体
	const buttonVariants = {
		initial: { scale: 1 },
		hover: {
			scale: disabled ? 1 : isMobile ? 1.01 : 1.02,
			transition: { duration: isMobile ? 0.1 : 0.2 },
		},
		tap: {
			scale: disabled ? 1 : isMobile ? 0.97 : 0.98,
			transition: { duration: isMobile ? 0.05 : 0.1 },
		},
	};

	// 样式类名
	const getVariantClasses = () => {
		switch (variant) {
			case "primary":
				return disabled
					? "bg-gray-300 text-gray-500 cursor-not-allowed"
					: "bg-blue-500 text-white hover:bg-blue-600 active:bg-blue-700";
			case "secondary":
				return disabled
					? "bg-gray-200 text-gray-400 cursor-not-allowed"
					: "bg-gray-200 text-gray-800 hover:bg-gray-300 active:bg-gray-400";
			case "ghost":
				return disabled
					? "text-gray-400 cursor-not-allowed"
					: "text-gray-600 hover:text-gray-800 hover:bg-gray-100 active:bg-gray-200";
			case "danger":
				return disabled
					? "bg-gray-300 text-gray-500 cursor-not-allowed"
					: "bg-red-500 text-white hover:bg-red-600 active:bg-red-700";
			default:
				return "";
		}
	};

	const getSizeClasses = () => {
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
	};

	return (
		<motion.button
			onClick={disabled ? undefined : onClick}
			variants={buttonVariants}
			initial="initial"
			whileHover="hover"
			whileTap="tap"
			title={title}
			className={`
        font-medium rounded-lg 
        transition-colors duration-200 
        focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500
        ${getVariantClasses()}
        ${getSizeClasses()}
        ${className}
        ${isMobile ? "touch-optimized mobile-optimized" : ""}
      `}
			style={{
				willChange: "transform",
				backfaceVisibility: "hidden",
				WebkitBackfaceVisibility: "hidden",
			}}
			{...motionProps}
		>
			{children}
		</motion.button>
	);
};

export default InteractiveButton;
