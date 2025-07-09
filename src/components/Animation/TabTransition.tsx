import { AnimatePresence, motion } from "framer-motion";
import { getTabDirection } from "../../hooks/useRouter";

interface TabTransitionProps {
	children: React.ReactNode;
	activeKey: string;
	direction?: "left" | "right";
	// 新增属性：用于动态方向检测
	previousTab?: string;
	tabGroup?: "accounting" | "timing";
}

// 标签切换动画变体 - 支持动态方向
const tabVariants = {
	initial: (animationDirection: "forward" | "backward") => ({
		opacity: 0,
		x: animationDirection === "forward" ? 50 : -50,
		scale: 0.98,
	}),
	in: {
		opacity: 1,
		x: 0,
		scale: 1,
	},
	out: (animationDirection: "forward" | "backward") => ({
		opacity: 0,
		x: animationDirection === "forward" ? -50 : 50,
		scale: 0.98,
	}),
};

const TabTransition: React.FC<TabTransitionProps> = ({
	children,
	activeKey,
	direction = "right",
	previousTab,
	tabGroup,
}) => {
	// 检测移动端并调整动画参数
	const isMobile = window.innerWidth < 768;

	// 计算动画方向
	const animationDirection =
		previousTab && tabGroup
			? getTabDirection(previousTab, activeKey, tabGroup)
			: "forward";

	// 如果无法检测方向，使用默认的 forward
	const finalAnimationDirection =
		animationDirection === "none" ? "forward" : animationDirection;

	const transition = {
		type: "spring" as const,
		stiffness: isMobile ? 500 : 400,
		damping: isMobile ? 40 : 35,
		duration: isMobile ? 0.15 : 0.2,
	};

	return (
		<AnimatePresence mode="wait" custom={finalAnimationDirection}>
			<motion.div
				key={activeKey}
				custom={finalAnimationDirection}
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
