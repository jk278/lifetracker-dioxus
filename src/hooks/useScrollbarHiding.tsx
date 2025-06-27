import { useEffect, useRef } from "react";

/**
 * 一个自定义 React Hook，用于在元素滚动时显示滚动条，
 * 并在滚动停止后的一段时间内自动隐藏滚动条。
 * @param {number} [hideDelay=1500] - 滚动停止后隐藏滚动条的延迟时间（毫秒）。
 * @returns {React.RefObject<T>} - 一个应附加到目标滚动元素的 ref 对象。
 */
export function useScrollbarHiding<T extends HTMLElement>(hideDelay = 1500) {
	const elementRef = useRef<T>(null);
	const hideTimerRef = useRef<number | null>(null);

	useEffect(() => {
		const element = elementRef.current;
		if (!element) return;

		const handleScroll = () => {
			// 当滚动时，为元素添加 'is-scrolling' 类
			element.classList.add("is-scrolling");

			// 清除之前的计时器，以防用户连续滚动
			if (hideTimerRef.current) {
				window.clearTimeout(hideTimerRef.current);
			}

			// 设置一个新的计时器，在滚动停止 hideDelay 毫秒后移除 'is-scrolling' 类
			hideTimerRef.current = window.setTimeout(() => {
				element.classList.remove("is-scrolling");
			}, hideDelay);
		};

		// 只监听滚动事件
		element.addEventListener("scroll", handleScroll, { passive: true });

		// 组件卸载时进行清理
		return () => {
			element.removeEventListener("scroll", handleScroll);
			if (hideTimerRef.current) {
				window.clearTimeout(hideTimerRef.current);
			}
		};
	}, [hideDelay]);

	return elementRef;
}
