import React, { createContext, useContext, useEffect, useState } from 'react';

type Theme = 'light' | 'dark' | 'system';

interface ThemeContextType {
    theme: Theme;
    setTheme: (theme: Theme) => void;
    resolvedTheme: 'light' | 'dark';
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export function ThemeProvider({ children }: { children: React.ReactNode }) {
    const [theme, setTheme] = useState<Theme>(() => {
        // 从本地存储获取主题设置，默认为 system
        if (typeof window !== 'undefined') {
            return (localStorage.getItem('theme') as Theme) || 'system';
        }
        return 'system';
    });

    const [resolvedTheme, setResolvedTheme] = useState<'light' | 'dark'>('light');

    // 获取系统主题
    const getSystemTheme = (): 'light' | 'dark' => {
        if (typeof window !== 'undefined' && window.matchMedia) {
            return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
        }
        return 'light';
    };

    // 应用主题到DOM
    const applyTheme = (appliedTheme: 'light' | 'dark') => {
        const root = window.document.documentElement;
        root.classList.remove('light', 'dark');
        root.classList.add(appliedTheme);
        setResolvedTheme(appliedTheme);
    };

    // 主题变化处理
    useEffect(() => {
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

        const handleSystemThemeChange = () => {
            if (theme === 'system') {
                const systemTheme = getSystemTheme();
                applyTheme(systemTheme);
            }
        };

        // 初始化主题
        let initialTheme: 'light' | 'dark';
        if (theme === 'system') {
            initialTheme = getSystemTheme();
        } else {
            initialTheme = theme;
        }
        applyTheme(initialTheme);

        // 监听系统主题变化
        mediaQuery.addEventListener('change', handleSystemThemeChange);

        // 保存到本地存储
        localStorage.setItem('theme', theme);

        return () => {
            mediaQuery.removeEventListener('change', handleSystemThemeChange);
        };
    }, [theme]);

    const value = {
        theme,
        setTheme,
        resolvedTheme,
    };

    return (
        <ThemeContext.Provider value={value}>
            {children}
        </ThemeContext.Provider>
    );
}

export function useTheme() {
    const context = useContext(ThemeContext);
    if (context === undefined) {
        throw new Error('useTheme must be used within a ThemeProvider');
    }
    return context;
} 