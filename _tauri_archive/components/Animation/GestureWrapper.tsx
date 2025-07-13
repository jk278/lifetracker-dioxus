import { motion, type PanInfo, useDragControls } from "framer-motion";
import { useCallback } from "react";

interface GestureWrapperProps {
	children: React.ReactNode;
	onSwipeLeft?: () => void;
	onSwipeRight?: () => void;
	onSwipeUp?: () => void;
	onSwipeDown?: () => void;
	swipeThreshold?: number;
	enableDrag?: boolean;
	dragConstraints?: {
		left?: number;
		right?: number;
		top?: number;
		bottom?: number;
	};
	className?: string;
}

const GestureWrapper: React.FC<GestureWrapperProps> = ({
	children,
	onSwipeLeft,
	onSwipeRight,
	onSwipeUp,
	onSwipeDown,
	swipeThreshold = 100,
	enableDrag = false,
	dragConstraints,
	className = "",
}) => {
	const dragControls = useDragControls();

	const handleDragEnd = useCallback(
		(_event: MouseEvent | TouchEvent | PointerEvent, info: PanInfo) => {
			const { offset, velocity } = info;
			const { x, y } = offset;

			// 检查是否达到滑动阈值或速度阈值
			const isSwipeX =
				Math.abs(x) > swipeThreshold || Math.abs(velocity.x) > 500;
			const isSwipeY =
				Math.abs(y) > swipeThreshold || Math.abs(velocity.y) > 500;

			if (isSwipeX && Math.abs(x) > Math.abs(y)) {
				// 水平滑动
				if (x > 0 && onSwipeRight) {
					onSwipeRight();
				} else if (x < 0 && onSwipeLeft) {
					onSwipeLeft();
				}
			} else if (isSwipeY && Math.abs(y) > Math.abs(x)) {
				// 垂直滑动
				if (y > 0 && onSwipeDown) {
					onSwipeDown();
				} else if (y < 0 && onSwipeUp) {
					onSwipeUp();
				}
			}
		},
		[onSwipeLeft, onSwipeRight, onSwipeUp, onSwipeDown, swipeThreshold],
	);

	return (
		<motion.div
			drag={enableDrag}
			dragControls={dragControls}
			dragConstraints={dragConstraints}
			dragElastic={0.1}
			onDragEnd={handleDragEnd}
			whileDrag={{ scale: 0.95 }}
			className={`touch-pan-y ${className}`}
			style={{
				willChange: "transform",
				touchAction: "pan-y",
			}}
		>
			{children}
		</motion.div>
	);
};

export default GestureWrapper;
