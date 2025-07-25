/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./src/**/*.{html,js,ts,jsx,tsx,rs}",
    "./assets/**/*.{html,js,ts,jsx,tsx}",
  ],
  darkMode: 'class', // 🔥 关键配置：使用class模式而不是media查询
  theme: {
    extend: {
      colors: {
        // 自定义主题颜色
        'theme-primary': 'var(--primary-color)',
        'theme-secondary': 'var(--secondary-color)',
        'theme-success': 'var(--success-color)',
        'theme-warning': 'var(--warning-color)',
        'theme-error': 'var(--error-color)',
        'theme-info': 'var(--info-color)',
      },
      backgroundColor: {
        'primary': 'var(--bg-primary)',
        'secondary': 'var(--bg-secondary)',
        'tertiary': 'var(--bg-tertiary)',
      },
      textColor: {
        'primary': 'var(--text-primary)',
        'secondary': 'var(--text-secondary)',
        'tertiary': 'var(--text-tertiary)',
      },
      borderColor: {
        'default': 'var(--border-color)',
      },
      boxShadow: {
        'sm-custom': 'var(--shadow-sm)',
        'md-custom': 'var(--shadow-md)',
        'lg-custom': 'var(--shadow-lg)',
      }
    },
  },
  plugins: [],
}