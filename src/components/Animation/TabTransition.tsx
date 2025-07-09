import { AnimatePresence, motion, type Transition } from "framer-motion";
import { useMemo } from "react";
import { getTabDirection } from "../../hooks/useRouter";

interface TabTransitionProps {
	children: React.ReactNode;
	activeKey: string;
	direction?: "left" | "right";
	// 新增属性：用于动态方向检测
	previousTab?: string;
	tabGroup?: "accounting" | "timing";
}

// 简化的标签切换动画变体 - 去除scale避免回弹
const tabVariants = {
	initial: (animationDirection: "forward" | "backward") => ({
		opacity: 0,
		x: animationDirection === "forward" ? 50 : -50,
		// 使用更平滑的blur效果替代scale
		filter: "blur(2px)",
	}),
	in: {
		opacity: 1,
		x: 0,
		filter: "blur(0px)",
	},
	out: (animationDirection: "forward" | "backward") => ({
		opacity: 0,
		x: animationDirection === "forward" ? -50 : 50,
		filter: "blur(2px)",
	}),
};

const TabTransition: React.FC<TabTransitionProps> = ({
	children,
	activeKey,
	direction = "right",
	previousTab,
	tabGroup,
}) => {
	// 使用useMemo优化性能
	const isMobile = useMemo(() => window.innerWidth < 768, []);

	// 计算动画方向
	const animationDirection = useMemo(() => {
		if (previousTab && tabGroup) {
			const detected = getTabDirection(previousTab, activeKey, tabGroup);
			return detected === "none" ? "forward" : detected;
		}
		return "forward";
	}, [previousTab, activeKey, tabGroup]);

	// 优化的transition配置
	const transition: Transition = useMemo(
		() => ({
			type: "tween",
			ease: [0.4, 0, 0.2, 1], // 使用更平滑的缓动函数
			duration: isMobile ? 0.2 : 0.25, // 移动端使用更短的持续时间
			bounce: 0, // 避免bounce效果
		}),
		[isMobile],
	);

	return (
		<AnimatePresence
			mode="wait"
			custom={animationDirection}
			// 添加onExitComplete回调
			onExitComplete={() => {
				// 清理可能残留的样式
				if (typeof window !== "undefined") {
					document.body.style.overflow = "";
				}
			}}
		>
			<motion.div
				key={activeKey}
				custom={animationDirection}
				initial="initial"
				animate="in"
				exit="out"
				variants={tabVariants}
				transition={transition}
				className="w-full h-full"
				style={{
					// 优化渲染性能
					willChange: "transform, opacity, filter",
					backfaceVisibility: "hidden",
					WebkitBackfaceVisibility: "hidden",
					// 改善移动端性能
					WebkitTransform: "translateZ(0)",
					transform: "translateZ(0)",
					// 避免layout shift
					contain: "layout style paint",
				}}
				// 添加动画事件监听
				onAnimationStart={() => {
					// 防止在标签切换时出现滚动问题
					if (typeof window !== "undefined") {
						document.body.style.overflow = "hidden";
					}
				}}
				onAnimationComplete={() => {
					// 恢复滚动
					if (typeof window !== "undefined") {
						document.body.style.overflow = "";
					}
				}}
			>
				{children}
			</motion.div>
		</AnimatePresence>
	);
};

export default TabTransition;
