import { AnimatePresence, motion, type Transition } from "framer-motion";
import { useMemo } from "react";

interface PageTransitionProps {
	children: React.ReactNode;
	routeKey: string;
	// The `custom` prop will be passed to AnimatePresence to control animation direction
	animationCustom: {
		direction: "horizontal" | "vertical";
		animationDirection: "forward" | "backward" | "none";
	};
	duration?: number;
}

// 简化的动画变体，去除scale以避免回弹
const slideVariants = {
	initial: (custom: PageTransitionProps["animationCustom"]) => {
		const { direction, animationDirection } = custom;

		// 简化动画属性，只使用位移和透明度
		const x =
			direction === "horizontal"
				? animationDirection === "forward"
					? 300
					: -300
				: 0;
		const y =
			direction === "vertical"
				? animationDirection === "forward"
					? 50
					: -50
				: 0;

		return {
			opacity: 0,
			x,
			y,
			// 使用更平滑的初始状态
			filter: "blur(4px)",
		};
	},
	in: {
		opacity: 1,
		x: 0,
		y: 0,
		filter: "blur(0px)",
	},
	out: (custom: PageTransitionProps["animationCustom"]) => {
		const { direction, animationDirection } = custom;

		const x =
			direction === "horizontal"
				? animationDirection === "forward"
					? -300
					: 300
				: 0;
		const y =
			direction === "vertical"
				? animationDirection === "forward"
					? -50
					: 50
				: 0;

		return {
			opacity: 0,
			x,
			y,
			filter: "blur(4px)",
		};
	},
};

const PageTransition: React.FC<PageTransitionProps> = ({
	children,
	routeKey,
	animationCustom,
	duration = 0.3,
}) => {
	// 使用useMemo优化性能
	const isMobile = useMemo(() => window.innerWidth < 768, []);
	const optimizedDuration = useMemo(
		() => (isMobile ? Math.min(duration, 0.2) : duration),
		[duration, isMobile],
	);

	// 优化的transition配置
	const transition: Transition = useMemo(
		() => ({
			type: "tween",
			ease: [0.4, 0, 0.2, 1], // 使用更平滑的缓动函数
			duration: optimizedDuration,
			// 避免bounce效果
			bounce: 0,
		}),
		[optimizedDuration],
	);

	return (
		<AnimatePresence
			mode="wait"
			custom={animationCustom}
			// 添加onExitComplete回调以确保动画完全结束
			onExitComplete={() => {
				// 清理可能残留的样式
				if (typeof window !== "undefined") {
					document.body.style.overflow = "";
				}
			}}
		>
			<motion.div
				key={routeKey}
				custom={animationCustom}
				initial="initial"
				animate="in"
				exit="out"
				variants={slideVariants}
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
					// 防止滚动条跳动
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

export default PageTransition;
