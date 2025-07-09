import { AnimatePresence, motion } from "framer-motion";

interface PageTransitionProps {
	children: React.ReactNode;
	routeKey: string;
	direction?: "horizontal" | "vertical";
	duration?: number;
}

// 页面切换动画变体
const slideVariants = {
	initial: (direction: "horizontal" | "vertical") => ({
		opacity: 0,
		x: direction === "horizontal" ? 300 : 0,
		y: direction === "vertical" ? 50 : 0,
		scale: 0.95,
	}),
	in: {
		opacity: 1,
		x: 0,
		y: 0,
		scale: 1,
	},
	out: (direction: "horizontal" | "vertical") => ({
		opacity: 0,
		x: direction === "horizontal" ? -300 : 0,
		y: direction === "vertical" ? -50 : 0,
		scale: 0.95,
	}),
};

const PageTransition: React.FC<PageTransitionProps> = ({
	children,
	routeKey,
	direction = "horizontal",
	duration = 0.3,
}) => {
	// 检测移动端并调整动画参数
	const isMobile = window.innerWidth < 768;
	const optimizedDuration = isMobile ? Math.min(duration, 0.2) : duration;

	const transition = {
		type: "spring" as const,
		stiffness: isMobile ? 400 : 300,
		damping: isMobile ? 35 : 30,
		duration: optimizedDuration,
	};

	return (
		<AnimatePresence mode="wait">
			<motion.div
				key={routeKey}
				custom={direction}
				initial="initial"
				animate="in"
				exit="out"
				variants={slideVariants}
				transition={transition}
				className={`w-full h-full ${isMobile ? "framer-motion-mobile touch-optimized" : ""}`}
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

export default PageTransition;
