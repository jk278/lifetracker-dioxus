// Tailwind CSS 暗色模式配置脚本
// 配置 Tailwind 使用 class 模式而不是 prefers-color-scheme
(function() {
    // 等待 Tailwind 加载完成
    if (typeof tailwind !== 'undefined') {
        // 如果 tailwind 全局对象可用，重新配置
        try {
            tailwind.config = {
                darkMode: 'class',
                ...tailwind.config
            };
        } catch (e) {
            console.warn('无法重新配置 Tailwind:', e);
        }
    }

    // 确保 dark 类的样式生效
    function ensureDarkModeStyles() {
        const html = document.documentElement;
        
        // 创建一个 MutationObserver 来监听 class 变化
        const observer = new MutationObserver(function(mutations) {
            mutations.forEach(function(mutation) {
                if (mutation.type === 'attributes' && mutation.attributeName === 'class') {
                    const hasDark = html.classList.contains('dark');
                    console.log('Dark mode class detected:', hasDark);
                    
                    // 强制触发重绘以确保样式应用
                    if (hasDark) {
                        html.style.colorScheme = 'dark';
                    } else {
                        html.style.colorScheme = 'light';
                    }
                }
            });
        });

        // 开始观察
        observer.observe(html, {
            attributes: true,
            attributeFilter: ['class']
        });

        // 初始检查
        if (html.classList.contains('dark')) {
            html.style.colorScheme = 'dark';
        }
    }

    // DOM 加载完成后执行
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', ensureDarkModeStyles);
    } else {
        ensureDarkModeStyles();
    }

    console.log('Tailwind 暗色模式配置脚本已加载');
})();