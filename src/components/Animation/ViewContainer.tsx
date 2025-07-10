import { AnimatePresence, motion } from "framer-motion";
import { useMemo } from "react";

interface ViewContainerProps {
	children: React.ReactNode;
	viewKey: string;
	viewType: "main" | "system-overview" | "system-detail";
	animationConfig?: {
		direction?: "forward" | "backward" | "none";
		type?: "slide" | "fade";
	};
}

// 主页面滑动动画（时间追踪、财务、笔记、系统）
const mainPageVariants = {
	initial: (direction: "forward" | "backward") => ({
		opacity: 0,
		x: direction === "forward" ? 300 : -300,
		filter: "blur(4px)",
	}),
	in: {
		opacity: 1,
		x: 0,
		filter: "blur(0px)",
	},
	out: (direction: "forward" | "backward") => ({
		opacity: 0,
		x: direction === "forward" ? -300 : 300,
		filter: "blur(4px)",
	}),
};

// 系统页面淡入动画（用于子页面）
const systemDetailVariants = {
	initial: {
		opacity: 0,
		scale: 0.95,
		filter: "blur(2px)",
	},
	in: {
		opacity: 1,
		scale: 1,
		filter: "blur(0px)",
	},
	out: {
		opacity: 0,
		scale: 0.95,
		filter: "blur(2px)",
	},
};

// 无动画变体（用于返回系统概览）
const staticVariants = {
	initial: { opacity: 1 },
	in: { opacity: 1 },
	out: { opacity: 1 },
};

const ViewContainer: React.FC<ViewContainerProps> = ({
	children,
	viewKey,
	viewType,
	animationConfig = {},
}) => {
	const { direction = "forward", type = "slide" } = animationConfig;

	// 根据视图类型选择动画变体
	const variants = useMemo(() => {
		switch (viewType) {
			case "main":
				return mainPageVariants;
			case "system-detail":
				return systemDetailVariants;
			case "system-overview":
			default:
				return staticVariants;
		}
	}, [viewType]);

	// 根据方向选择过渡配置
	const transition = useMemo(() => {
		if (direction === "none") return { duration: 0 };

		const baseDuration = viewType === "main" ? 0.3 : 0.25;
		return {
			type: "tween",
			ease: [0.4, 0, 0.2, 1],
			duration: baseDuration,
			bounce: 0,
		};
	}, [direction, viewType]);

	// 如果是无动画模式，直接渲染
	if (direction === "none") {
		return <div className="w-full h-full">{children}</div>;
	}

	return (
		<AnimatePresence mode="wait" initial={false}>
			<motion.div
				key={viewKey}
				custom={direction}
				initial="initial"
				animate="in"
				exit="out"
				variants={variants}
				transition={transition}
				className="w-full h-full"
				style={{
					willChange: "transform, opacity, filter",
					backfaceVisibility: "hidden",
					WebkitBackfaceVisibility: "hidden",
					WebkitTransform: "translateZ(0)",
					transform: "translateZ(0)",
					contain: "layout style paint",
				}}
			>
				{children}
			</motion.div>
		</AnimatePresence>
	);
};

export default ViewContainer;
