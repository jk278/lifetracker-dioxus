import { AnimatePresence, motion, type Transition } from "framer-motion";

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

// Page transition variants that react to the `custom` prop
const slideVariants = {
	initial: (custom: PageTransitionProps["animationCustom"]) => {
		const { direction, animationDirection } = custom;
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
		return { opacity: 0, x, y, scale: 0.95 };
	},
	in: {
		opacity: 1,
		x: 0,
		y: 0,
		scale: 1,
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
		return { opacity: 0, x, y, scale: 0.95 };
	},
};

const PageTransition: React.FC<PageTransitionProps> = ({
	children,
	routeKey,
	animationCustom,
	duration = 0.3,
}) => {
	const isMobile = window.innerWidth < 768;
	const optimizedDuration = isMobile ? Math.min(duration, 0.2) : duration;

	const transition: Transition = {
		type: "tween",
		ease: "easeOut",
		duration: optimizedDuration,
	};

	return (
		<AnimatePresence mode="wait" custom={animationCustom}>
			<motion.div
				key={routeKey}
				custom={animationCustom}
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
