import { AnimatePresence, motion, type Transition } from "framer-motion";
import { useMemo } from "react";

interface PageTransitionProps {
	children: React.ReactNode;
	routeKey: string;
	// The `custom` prop will be passed to AnimatePresence to control animation direction
	animationCustom: {
		direction: "horizontal" | "vertical";
		animationDirection: "forward" | "backward" | "none";
		// 新增：动画类型
		type?: "slide" | "fade";
		// 新增：只显示退出动画，不显示进入动画
		exitOnly?: boolean;
		// 新增：跳过退出动画，直接消失
		skipExitAnimation?: boolean;
		// 新增：退出动画延迟时间（秒）
		exitDelay?: number;
	};
	duration?: number;
}

// 滑动动画变体
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
			// 退出动画专用的 transition 配置
			transition: {
				type: "tween" as const,
				ease: [0.4, 0, 0.2, 1] as const,
				duration: custom.skipExitAnimation ? 0.05 : 0.3,
				delay: custom.exitDelay ?? 0,
				bounce: 0,
			},
		};
	},
};

// 淡入淡出动画变体 - 用于子页面（更明显的缩放与透明度）
const fadeVariants = {
	initial: {
		opacity: 0,
		scale: 0.9, // 更明显的缩放
		filter: "blur(4px)",
	},
	in: {
		opacity: 1,
		scale: 1,
		filter: "blur(0px)",
	},
	out: (custom: PageTransitionProps["animationCustom"]) => ({
		opacity: 0,
		scale: 0.9, // 与initial保持一致
		filter: "blur(4px)",
		// 退出动画专用的 transition 配置
		transition: {
			type: "tween" as const,
			ease: [0.4, 0, 0.2, 1] as const,
			duration: custom.skipExitAnimation ? 0.05 : 0.24, // fade动画稍快一些
			delay: custom.exitDelay ?? 0,
			bounce: 0,
		},
	}),
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

	// 根据动画类型选择变体
	const variants = useMemo(() => {
		const baseVariants =
			animationCustom.type === "fade" ? fadeVariants : slideVariants;

		// 如果是exitOnly模式，修改initial状态为直接显示
		if (animationCustom.exitOnly) {
			return {
				...baseVariants,
				initial: baseVariants.in, // 直接显示，不使用进入动画
			};
		}

		return baseVariants;
	}, [animationCustom.type, animationCustom.exitOnly]);

	// 优化的transition配置 - 仅用于进入动画
	const transition: Transition = useMemo(() => {
		const baseTransition = {
			type: "tween" as const,
			ease: [0.4, 0, 0.2, 1] as const,
			duration:
				animationCustom.type === "fade" ? optimizedDuration * 0.8 : optimizedDuration,
			bounce: 0,
		};

		return baseTransition;
	}, [
		optimizedDuration,
		animationCustom.type,
	]);

	// 如果动画方向是"none"，直接返回内容，不使用动画
	if (animationCustom.animationDirection === "none") {
		return <div className="w-full h-full">{children}</div>;
	}

	return (
		<AnimatePresence
			mode="wait"
			custom={animationCustom}
			initial={false}
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
				variants={variants}
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
