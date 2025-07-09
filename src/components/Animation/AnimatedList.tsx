import { AnimatePresence, motion } from "framer-motion";
import React from "react";

interface AnimatedListProps {
	children: React.ReactNode[];
	className?: string;
	itemClassName?: string;
	staggerDelay?: number;
	animation?: "fade" | "slide" | "scale";
}

const AnimatedList: React.FC<AnimatedListProps> = ({
	children,
	className = "",
	itemClassName = "",
	staggerDelay = 0.1,
	animation = "slide",
}) => {
	// 检测移动端并调整动画参数
	const isMobile = window.innerWidth < 768;
	// 根据动画类型定义变体
	const getItemVariants = () => {
		switch (animation) {
			case "fade":
				return {
					initial: { opacity: 0 },
					animate: { opacity: 1 },
					exit: { opacity: 0 },
				};
			case "scale":
				return {
					initial: { opacity: 0, scale: 0.8 },
					animate: { opacity: 1, scale: 1 },
					exit: { opacity: 0, scale: 0.8 },
				};
			case "slide":
			default:
				return {
					initial: { opacity: 0, y: 20 },
					animate: { opacity: 1, y: 0 },
					exit: { opacity: 0, x: -300 },
				};
		}
	};

	const itemVariants = getItemVariants();

	const containerVariants = {
		animate: {
			transition: {
				staggerChildren: isMobile
					? Math.max(staggerDelay * 0.5, 0.05)
					: staggerDelay,
			},
		},
	};

	// 使用 React.Children.toArray 统一处理 children，保留其原有 key
	const childrenArray = React.Children.toArray(children);

	return (
		<motion.div
			variants={containerVariants}
			initial="initial"
			animate="animate"
			className={`space-y-2 ${className}`}
		>
			<AnimatePresence mode="popLayout">
				{childrenArray.map((child) => (
					<motion.div
						key={(child as React.ReactElement).key as React.Key}
						variants={itemVariants}
						initial="initial"
						animate="animate"
						exit="exit"
						layout
						transition={{
							type: "spring" as const,
							stiffness: isMobile ? 500 : 400,
							damping: isMobile ? 35 : 30,
							duration: isMobile ? 0.2 : 0.3,
						}}
						className={`${itemClassName} ${isMobile ? "framer-motion-mobile mobile-optimized" : ""}`}
						style={{
							willChange: "transform, opacity",
							backfaceVisibility: "hidden",
							WebkitBackfaceVisibility: "hidden",
						}}
					>
						{child}
					</motion.div>
				))}
			</AnimatePresence>
		</motion.div>
	);
};

export default AnimatedList;
