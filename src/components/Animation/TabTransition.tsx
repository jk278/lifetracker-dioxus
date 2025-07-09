import { AnimatePresence, motion } from "framer-motion";

interface TabTransitionProps {
	children: React.ReactNode;
	activeKey: string;
	direction?: "left" | "right";
}

// 标签切换动画变体
const tabVariants = {
	initial: (direction: "left" | "right") => ({
		opacity: 0,
		x: direction === "right" ? 50 : -50,
		scale: 0.98,
	}),
	in: {
		opacity: 1,
		x: 0,
		scale: 1,
	},
	out: (direction: "left" | "right") => ({
		opacity: 0,
		x: direction === "right" ? -50 : 50,
		scale: 0.98,
	}),
};

const TabTransition: React.FC<TabTransitionProps> = ({
	children,
	activeKey,
	direction = "right",
}) => {
	// 检测移动端并调整动画参数
	const isMobile = window.innerWidth < 768;

	const transition = {
		type: "spring" as const,
		stiffness: isMobile ? 500 : 400,
		damping: isMobile ? 40 : 35,
		duration: isMobile ? 0.15 : 0.2,
	};

	return (
		<AnimatePresence mode="wait">
			<motion.div
				key={activeKey}
				custom={direction}
				initial="initial"
				animate="in"
				exit="out"
				variants={tabVariants}
				transition={transition}
				className={`w-full h-full ${isMobile ? "framer-motion-mobile mobile-optimized" : ""}`}
				style={{
					willChange: "transform, opacity",
					backfaceVisibility: "hidden",
					WebkitBackfaceVisibility: "hidden",
				}}
			>
				{children}
			</motion.div>
		</AnimatePresence>
	);
};

export default TabTransition;
