import { invoke } from "@tauri-apps/api/core";
import type React from "react";
import {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useState,
} from "react";

type Theme = "light" | "dark" | "system";

// Material You 风格主题色配置
export const THEME_COLORS = {
	blue: {
		name: "蓝色",
		primary: "blue",
		colors: {
			50: "#e3f2fd",
			100: "#bbdefb",
			500: "#2196f3",
			600: "#1976d2",
			700: "#1565c0",
			900: "#0d47a1",
			// Material You 背景色
			lightBg: "#fafcff",
			darkBg: "#0f1419",
			lightSurface: "#f8fafe",
			darkSurface: "#151b20",
		},
	},
	green: {
		name: "绿色",
		primary: "green",
		colors: {
			50: "#e8f5e8",
			100: "#c8e6c9",
			500: "#4caf50",
			600: "#43a047",
			700: "#388e3c",
			900: "#1b5e20",
			lightBg: "#fafff8",
			darkBg: "#0f1a0e",
			lightSurface: "#f4fff2",
			darkSurface: "#141f13",
		},
	},
	purple: {
		name: "紫色",
		primary: "purple",
		colors: {
			50: "#f3e5f5",
			100: "#e1bee7",
			500: "#9c27b0",
			600: "#8e24aa",
			700: "#7b1fa2",
			900: "#4a148c",
			lightBg: "#fefaff",
			darkBg: "#1a0f1a",
			lightSurface: "#fcf7ff",
			darkSurface: "#201420",
		},
	},
	red: {
		name: "红色",
		primary: "red",
		colors: {
			50: "#ffebee",
			100: "#ffcdd2",
			500: "#f44336",
			600: "#e53935",
			700: "#d32f2f",
			900: "#b71c1c",
			lightBg: "#fffafa",
			darkBg: "#1a0f0f",
			lightSurface: "#fff8f8",
			darkSurface: "#201414",
		},
	},
	orange: {
		name: "橙色",
		primary: "orange",
		colors: {
			50: "#fff3e0",
			100: "#ffe0b2",
			500: "#ff9800",
			600: "#fb8c00",
			700: "#f57c00",
			900: "#e65100",
			lightBg: "#fffcf9",
			darkBg: "#1a140f",
			lightSurface: "#fffaf6",
			darkSurface: "#201814",
		},
	},
	teal: {
		name: "青绿",
		primary: "teal",
		colors: {
			50: "#e0f2f1",
			100: "#b2dfdb",
			500: "#009688",
			600: "#00897b",
			700: "#00796b",
			900: "#004d40",
			lightBg: "#f9ffff",
			darkBg: "#0f1a19",
			lightSurface: "#f4fffe",
			darkSurface: "#132019",
		},
	},
	pink: {
		name: "粉色",
		primary: "pink",
		colors: {
			50: "#fce4ec",
			100: "#f8bbd9",
			500: "#e91e63",
			600: "#d81b60",
			700: "#c2185b",
			900: "#880e4f",
			lightBg: "#fffafc",
			darkBg: "#1a0f14",
			lightSurface: "#fff6f9",
			darkSurface: "#201318",
		},
	},
	indigo: {
		name: "靛蓝",
		primary: "indigo",
		colors: {
			50: "#e8eaf6",
			100: "#c5cae9",
			500: "#3f51b5",
			600: "#3949ab",
			700: "#303f9f",
			900: "#1a237e",
			lightBg: "#fafbff",
			darkBg: "#0f1119",
			lightSurface: "#f7f8ff",
			darkSurface: "#14151f",
		},
	},
} as const;

export type ThemeColor = keyof typeof THEME_COLORS;

interface ThemeContextType {
	theme: Theme;
	setTheme: (theme: Theme) => void;
	resolvedTheme: "light" | "dark";
	themeColor: ThemeColor;
	setThemeColor: (color: ThemeColor) => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

// 颜色工具函数: 十六进制 <-> RGB 互转，并实现颜色混合(线性插值)
function hexToRgb(hex: string): [number, number, number] {
	let normalized = hex.replace("#", "");
	if (normalized.length === 3) {
		normalized = normalized
			.split("")
			.map((c) => c + c)
			.join("");
	}
	const bigint = Number.parseInt(normalized, 16);
	const r = (bigint >> 16) & 255;
	const g = (bigint >> 8) & 255;
	const b = bigint & 255;
	return [r, g, b];
}

function rgbToHex(r: number, g: number, b: number): string {
	return (
		"#" +
		[r, g, b]
			.map((x) => Math.max(0, Math.min(255, x)).toString(16).padStart(2, "0"))
			.join("")
	);
}

/**
 * 将 color2 按 weight (0~1) 比例混入 color1，返回新的十六进制颜色
 * weight = 0 表示完全使用 color1，1 表示完全使用 color2
 */
function mixColors(color1: string, color2: string, weight: number): string {
	const [r1, g1, b1] = hexToRgb(color1);
	const [r2, g2, b2] = hexToRgb(color2);
	const r = Math.round(r1 * (1 - weight) + r2 * weight);
	const g = Math.round(g1 * (1 - weight) + g2 * weight);
	const b = Math.round(b1 * (1 - weight) + b2 * weight);
	return rgbToHex(r, g, b);
}

export function ThemeProvider({ children }: { children: React.ReactNode }) {
	// 清理无效的localStorage数据
	useEffect(() => {
		if (typeof window !== "undefined") {
			const savedColor = localStorage.getItem("themeColor");
			if (savedColor && !THEME_COLORS[savedColor as ThemeColor]) {
				console.warn(
					`Invalid theme color ${savedColor} found in localStorage, clearing...`,
				);
				localStorage.removeItem("themeColor");
			}
		}
	}, []);

	const [theme, setTheme] = useState<Theme>(() => {
		// 从本地存储获取主题设置，默认为 system
		if (typeof window !== "undefined") {
			return (localStorage.getItem("theme") as Theme) || "system";
		}
		return "system";
	});

	const [themeColor, setThemeColor] = useState<ThemeColor>(() => {
		// 从本地存储获取主题色设置，默认为蓝色
		if (typeof window !== "undefined") {
			const savedColor = localStorage.getItem("themeColor") as ThemeColor;
			// 验证保存的颜色是否在当前配置中存在
			if (savedColor && THEME_COLORS[savedColor]) {
				return savedColor;
			}
		}
		return "blue";
	});

	const [resolvedTheme, setResolvedTheme] = useState<"light" | "dark">("light");

	// 获取系统主题
	const getSystemTheme = useCallback((): "light" | "dark" => {
		if (typeof window !== "undefined" && window.matchMedia) {
			return window.matchMedia("(prefers-color-scheme: dark)").matches
				? "dark"
				: "light";
		}
		return "light";
	}, []);

	// 应用主题色和背景色CSS变量
	const applyThemeColorVariables = useCallback(
		(colorKey: ThemeColor, isDark: boolean) => {
			const colorConfig = THEME_COLORS[colorKey];
			if (!colorConfig) {
				console.warn(`Theme color ${colorKey} not found, falling back to blue`);
				return applyThemeColorVariables("blue", isDark);
			}
			const root = window.document.documentElement;

			// 设置主题色CSS变量
			root.style.setProperty("--theme-primary", colorConfig.colors[500]);
			root.style.setProperty("--theme-primary-hover", colorConfig.colors[600]);
			root.style.setProperty("--theme-primary-active", colorConfig.colors[700]);
			root.style.setProperty("--theme-primary-light", colorConfig.colors[50]);
			root.style.setProperty(
				"--theme-primary-lighter",
				colorConfig.colors[100],
			);
			root.style.setProperty("--theme-primary-dark", colorConfig.colors[900]);

			// 动态计算背景色: 99% 黑/白 + 1% 主题色 (更深/更浅)
			const primaryColor = colorConfig.colors[500];
			if (isDark) {
				const darkBg = mixColors("#000000", primaryColor, 0.02);
				const darkSurface = mixColors("#000000", primaryColor, 0.08);
				root.style.setProperty("--theme-background", darkBg);
				root.style.setProperty("--theme-surface", darkSurface);
			} else {
				const lightBg = mixColors("#ffffff", primaryColor, 0.02);
				const lightSurface = mixColors("#ffffff", primaryColor, 0.08);
				root.style.setProperty("--theme-background", lightBg);
				root.style.setProperty("--theme-surface", lightSurface);
			}

			// 存储到localStorage
			localStorage.setItem("themeColor", colorKey);
		},
		[],
	);

	// 应用主题到DOM和窗口背景
	const applyTheme = useCallback(
		async (appliedTheme: "light" | "dark") => {
			const root = window.document.documentElement;
			root.classList.remove("light", "dark");
			root.classList.add(appliedTheme);
			setResolvedTheme(appliedTheme);

			// 应用主题色变量（包括背景色）
			applyThemeColorVariables(themeColor, appliedTheme === "dark");

			// 同步更新窗口背景色，避免拖拽残影
			try {
				await invoke("set_window_theme", { isDark: appliedTheme === "dark" });
			} catch (error) {
				console.warn("更新窗口背景色失败:", error);
			}
		},
		[themeColor, applyThemeColorVariables],
	);

	// 主题变化处理
	useEffect(() => {
		const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

		const handleSystemThemeChange = () => {
			if (theme === "system") {
				const systemTheme = getSystemTheme();
				applyTheme(systemTheme);
			}
		};

		// 初始化主题
		let initialTheme: "light" | "dark";
		if (theme === "system") {
			initialTheme = getSystemTheme();
		} else {
			initialTheme = theme;
		}
		applyTheme(initialTheme);

		// 监听系统主题变化
		mediaQuery.addEventListener("change", handleSystemThemeChange);

		// 保存到本地存储
		localStorage.setItem("theme", theme);

		return () => {
			mediaQuery.removeEventListener("change", handleSystemThemeChange);
		};
	}, [theme, applyTheme, getSystemTheme]);

	// 主题色变化处理
	useEffect(() => {
		applyThemeColorVariables(themeColor, resolvedTheme === "dark");
	}, [themeColor, resolvedTheme, applyThemeColorVariables]);

	const value = {
		theme,
		setTheme,
		resolvedTheme,
		themeColor,
		setThemeColor,
	};

	return (
		<ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>
	);
}

export function useTheme() {
	const context = useContext(ThemeContext);
	if (context === undefined) {
		throw new Error("useTheme must be used within a ThemeProvider");
	}
	return context;
}
