//! # GUI主题模块
//!
//! 定义TimeTracker应用程序的视觉主题和样式

use eframe::egui;
use serde::{Deserialize, Serialize};

/// 应用程序主题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// 是否为深色模式
    pub dark_mode: bool,
    /// 主色调
    pub primary_color: ColorScheme,
    /// 字体配置
    pub fonts: FontConfig,
    /// 间距配置
    pub spacing: SpacingConfig,
    /// 动画配置
    pub animations: AnimationConfig,
}

/// 颜色方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    /// 主要颜色
    pub primary: [u8; 3],
    /// 次要颜色
    pub secondary: [u8; 3],
    /// 强调色
    pub accent: [u8; 3],
    /// 成功色
    pub success: [u8; 3],
    /// 警告色
    pub warning: [u8; 3],
    /// 错误色
    pub error: [u8; 3],
    /// 信息色
    pub info: [u8; 3],
}

/// 字体配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    /// 默认字体大小
    pub default_size: f32,
    /// 标题字体大小
    pub heading_size: f32,
    /// 小字体大小
    pub small_size: f32,
    /// 代码字体大小
    pub code_size: f32,
    /// 字体族
    pub font_family: String,
    /// 代码字体族
    pub code_font_family: String,
}

/// 间距配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingConfig {
    /// 默认间距
    pub default: f32,
    /// 小间距
    pub small: f32,
    /// 大间距
    pub large: f32,
    /// 按钮内边距
    pub button_padding: [f32; 2],
    /// 窗口内边距
    pub window_margin: f32,
    /// 面板间距
    pub panel_margin: f32,
}

/// 动画配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// 是否启用动画
    pub enabled: bool,
    /// 动画持续时间（毫秒）
    pub duration_ms: u64,
    /// 缓动函数类型
    pub easing: EasingType,
}

/// 缓动函数类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

/// 预定义主题
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemePreset {
    Default,
    Light,
    Dark,
    Blue,
    Green,
    Purple,
    Orange,
}

impl Default for Theme {
    fn default() -> Self {
        Self::light_theme()
    }
}

impl Theme {
    /// 创建浅色主题
    pub fn light_theme() -> Self {
        Self {
            dark_mode: false,
            primary_color: ColorScheme {
                primary: [70, 130, 180],    // 钢蓝色
                secondary: [176, 196, 222], // 浅钢蓝色
                accent: [255, 140, 0],      // 深橙色
                success: [34, 139, 34],     // 森林绿
                warning: [255, 165, 0],     // 橙色
                error: [220, 20, 60],       // 深红色
                info: [30, 144, 255],       // 道奇蓝
            },
            fonts: FontConfig {
                default_size: 14.0,
                heading_size: 18.0,
                small_size: 12.0,
                code_size: 13.0,
                font_family: "微软雅黑".to_string(),
                code_font_family: "Consolas".to_string(),
            },
            spacing: SpacingConfig {
                default: 8.0,
                small: 4.0,
                large: 16.0,
                button_padding: [8.0, 4.0],
                window_margin: 8.0,
                panel_margin: 4.0,
            },
            animations: AnimationConfig {
                enabled: true,
                duration_ms: 200,
                easing: EasingType::EaseOut,
            },
        }
    }

    /// 创建深色主题
    pub fn dark_theme() -> Self {
        let mut theme = Self::light_theme();
        theme.dark_mode = true;
        theme.primary_color = ColorScheme {
            primary: [100, 149, 237],   // 矢车菊蓝
            secondary: [119, 136, 153], // 浅石板灰
            accent: [255, 165, 0],      // 橙色
            success: [50, 205, 50],     // 酸橙绿
            warning: [255, 215, 0],     // 金色
            error: [255, 99, 71],       // 番茄色
            info: [135, 206, 235],      // 天空蓝
        };
        theme
    }

    /// 从预设创建主题
    pub fn from_preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::Default | ThemePreset::Light => Self::light_theme(),
            ThemePreset::Dark => Self::dark_theme(),
            ThemePreset::Blue => {
                let mut theme = Self::light_theme();
                let blue_color = egui::Color32::from_rgb(65, 105, 225);
                theme.primary_color = ColorScheme::from_color32(blue_color);
                theme.primary_color.accent = [30, 144, 255]; // 道奇蓝
                theme
            }
            ThemePreset::Green => {
                let mut theme = Self::light_theme();
                let green_color = egui::Color32::from_rgb(34, 139, 34);
                theme.primary_color = ColorScheme::from_color32(green_color);
                theme.primary_color.accent = [50, 205, 50]; // 酸橙绿
                theme
            }
            ThemePreset::Purple => {
                let mut theme = Self::light_theme();
                let purple_color = egui::Color32::from_rgb(138, 43, 226);
                theme.primary_color = ColorScheme::from_color32(purple_color);
                theme.primary_color.accent = [186, 85, 211]; // 中兰花紫
                theme
            }
            ThemePreset::Orange => {
                let mut theme = Self::light_theme();
                let orange_color = egui::Color32::from_rgb(255, 140, 0);
                theme.primary_color = ColorScheme::from_color32(orange_color);
                theme.primary_color.accent = [255, 165, 0]; // 橙色
                theme
            }
        }
    }

    /// 切换深色/浅色模式
    pub fn toggle_dark_mode(&mut self) {
        self.dark_mode = !self.dark_mode;

        // 调整颜色以适应新模式
        if self.dark_mode {
            // 切换到深色模式时调整颜色
            self.adjust_colors_for_dark_mode();
        } else {
            // 切换到浅色模式时调整颜色
            self.adjust_colors_for_light_mode();
        }
    }

    /// 为深色模式调整颜色
    fn adjust_colors_for_dark_mode(&mut self) {
        // 增加颜色亮度
        self.primary_color.primary = self.brighten_color(self.primary_color.primary, 30);
        self.primary_color.secondary = self.brighten_color(self.primary_color.secondary, 20);
        self.primary_color.success = self.brighten_color(self.primary_color.success, 40);
        self.primary_color.info = self.brighten_color(self.primary_color.info, 30);
    }

    /// 为浅色模式调整颜色
    fn adjust_colors_for_light_mode(&mut self) {
        // 降低颜色亮度
        self.primary_color.primary = self.darken_color(self.primary_color.primary, 30);
        self.primary_color.secondary = self.darken_color(self.primary_color.secondary, 20);
        self.primary_color.success = self.darken_color(self.primary_color.success, 40);
        self.primary_color.info = self.darken_color(self.primary_color.info, 30);
    }

    /// 增加颜色亮度
    fn brighten_color(&self, color: [u8; 3], amount: u8) -> [u8; 3] {
        [
            (color[0] as u16 + amount as u16).min(255) as u8,
            (color[1] as u16 + amount as u16).min(255) as u8,
            (color[2] as u16 + amount as u16).min(255) as u8,
        ]
    }

    /// 降低颜色亮度
    fn darken_color(&self, color: [u8; 3], amount: u8) -> [u8; 3] {
        [
            color[0].saturating_sub(amount),
            color[1].saturating_sub(amount),
            color[2].saturating_sub(amount),
        ]
    }

    /// 应用主题到egui上下文
    pub fn apply(&self, ctx: &egui::Context) {
        // 设置视觉样式
        let mut visuals = if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        // 应用自定义颜色
        self.apply_colors(&mut visuals);

        // 应用间距
        self.apply_spacing(&mut visuals);

        // 设置视觉样式
        ctx.set_visuals(visuals);

        // 设置字体
        self.apply_fonts(ctx);
    }

    /// 应用颜色到视觉样式
    fn apply_colors(&self, visuals: &mut egui::Visuals) {
        let primary = egui::Color32::from_rgb(
            self.primary_color.primary[0],
            self.primary_color.primary[1],
            self.primary_color.primary[2],
        );

        let accent = egui::Color32::from_rgb(
            self.primary_color.accent[0],
            self.primary_color.accent[1],
            self.primary_color.accent[2],
        );

        // 设置选中状态的颜色 - 确保在两种模式下都有合适的对比度
        let selection_color = if self.dark_mode {
            // 深色模式：使用较暗的选中色，确保白色文字可见
            egui::Color32::from_rgba_unmultiplied(
                (primary.r() as f32 * 0.4) as u8,
                (primary.g() as f32 * 0.4) as u8,
                (primary.b() as f32 * 0.4) as u8,
                180,
            )
        } else {
            // 亮色模式：使用更明显的主色调，确保有足够的颜色对比度
            egui::Color32::from_rgba_unmultiplied(
                (primary.r() as f32 * 0.6 + 255.0 * 0.4) as u8,
                (primary.g() as f32 * 0.6 + 255.0 * 0.4) as u8,
                (primary.b() as f32 * 0.6 + 255.0 * 0.4) as u8,
                120,
            )
        };

        visuals.selection.bg_fill = selection_color;
        visuals.selection.stroke.color = primary;

        // 设置强调色
        visuals.hyperlink_color = accent;

        // 设置按钮颜色 - 确保文字可见
        visuals.widgets.inactive.bg_fill = primary.gamma_multiply(0.8);

        // 悬停状态：使用稍微亮一点的颜色，但保持半透明
        let hovered_color = egui::Color32::from_rgba_unmultiplied(
            (primary.r() as f32 * 1.1).min(255.0) as u8,
            (primary.g() as f32 * 1.1).min(255.0) as u8,
            (primary.b() as f32 * 1.1).min(255.0) as u8,
            180, // 稍微透明，确保文字可见
        );

        // 激活状态：使用更亮的颜色，但仍然保持一定透明度
        let active_color = egui::Color32::from_rgba_unmultiplied(
            (primary.r() as f32 * 1.2).min(255.0) as u8,
            (primary.g() as f32 * 1.2).min(255.0) as u8,
            (primary.b() as f32 * 1.2).min(255.0) as u8,
            200, // 较少透明，但仍确保文字可见
        );

        visuals.widgets.hovered.bg_fill = hovered_color;
        visuals.widgets.active.bg_fill = active_color;

        // 注意：在egui中，text_color是方法而不是字段，无法直接设置
        // 文字颜色主要通过背景色的对比度来保证可见性

        // 设置错误和警告颜色
        visuals.error_fg_color = egui::Color32::from_rgb(
            self.primary_color.error[0],
            self.primary_color.error[1],
            self.primary_color.error[2],
        );

        visuals.warn_fg_color = egui::Color32::from_rgb(
            self.primary_color.warning[0],
            self.primary_color.warning[1],
            self.primary_color.warning[2],
        );
    }

    /// 应用间距到视觉样式
    fn apply_spacing(&self, visuals: &mut egui::Visuals) {
        // 设置按钮内边距
        visuals.button_frame = true;

        // 设置窗口圆角
        visuals.window_rounding = egui::Rounding::same(4.0);
        visuals.menu_rounding = egui::Rounding::same(4.0);

        // 设置控件圆角
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(2.0);
        visuals.widgets.inactive.rounding = egui::Rounding::same(2.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(2.0);
        visuals.widgets.active.rounding = egui::Rounding::same(2.0);
        visuals.widgets.open.rounding = egui::Rounding::same(2.0);
    }

    /// 应用字体到上下文
    fn apply_fonts(&self, ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // 为中文字体添加系统字体
        // 在Windows上，这些字体通常可用
        #[cfg(target_os = "windows")]
        {
            // 尝试加载Windows系统字体
            if let Some(font_data) = Self::load_system_font_windows() {
                fonts.font_data.insert("chinese".to_owned(), font_data);

                // 将中文字体添加到字体族的开头，确保优先使用
                fonts
                    .families
                    .get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .insert(0, "chinese".to_owned());

                fonts
                    .families
                    .get_mut(&egui::FontFamily::Monospace)
                    .unwrap()
                    .insert(0, "chinese".to_owned());
            }
        }

        // 为其他系统添加后备方案
        #[cfg(not(target_os = "windows"))]
        {
            // Linux和macOS的中文字体路径
            if let Some(font_data) = Self::load_system_font_unix() {
                fonts.font_data.insert("chinese".to_owned(), font_data);

                fonts
                    .families
                    .get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .insert(0, "chinese".to_owned());

                fonts
                    .families
                    .get_mut(&egui::FontFamily::Monospace)
                    .unwrap()
                    .insert(0, "chinese".to_owned());
            }
        }

        ctx.set_fonts(fonts);

        // 设置样式
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::new(self.fonts.heading_size, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(self.fonts.default_size, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::new(self.fonts.code_size, egui::FontFamily::Monospace),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(self.fonts.default_size, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Small,
                egui::FontId::new(self.fonts.small_size, egui::FontFamily::Proportional),
            ),
        ]
        .into();

        // 设置间距
        style.spacing.item_spacing = egui::Vec2::splat(self.spacing.default);
        style.spacing.button_padding = egui::Vec2::new(
            self.spacing.button_padding[0],
            self.spacing.button_padding[1],
        );
        style.spacing.window_margin = egui::Margin::same(self.spacing.window_margin);
        style.spacing.menu_margin = egui::Margin::same(self.spacing.panel_margin);

        ctx.set_style(style);
    }

    /// 加载Windows系统字体
    #[cfg(target_os = "windows")]
    fn load_system_font_windows() -> Option<egui::FontData> {
        // 尝试加载常见的中文字体
        let font_paths = [
            "C:\\Windows\\Fonts\\msyh.ttc",    // 微软雅黑
            "C:\\Windows\\Fonts\\simsun.ttc",  // 宋体
            "C:\\Windows\\Fonts\\simhei.ttf",  // 黑体
            "C:\\Windows\\Fonts\\simkai.ttf",  // 楷体
            "C:\\Windows\\Fonts\\simfang.ttf", // 仿宋
        ];

        for path in &font_paths {
            if let Ok(font_data) = std::fs::read(path) {
                return Some(egui::FontData::from_owned(font_data));
            }
        }

        None
    }

    /// 加载Unix系统字体（Linux/macOS）
    #[cfg(not(target_os = "windows"))]
    fn load_system_font_unix() -> Option<egui::FontData> {
        // 尝试加载常见的中文字体路径
        let font_paths = [
            // macOS
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/STHeiti Medium.ttc",
            // Linux
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            "/usr/share/fonts/truetype/arphic/uming.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ];

        for path in &font_paths {
            if let Ok(font_data) = std::fs::read(path) {
                return Some(egui::FontData::from_owned(font_data));
            }
        }

        None
    }

    /// 获取主题颜色
    pub fn get_color(&self, color_type: ColorType) -> egui::Color32 {
        match color_type {
            ColorType::Primary => self.primary_color.to_color32(),
            ColorType::Secondary => {
                let color = self.primary_color.secondary;
                egui::Color32::from_rgb(color[0], color[1], color[2])
            }
            ColorType::Accent => {
                let color = self.primary_color.accent;
                egui::Color32::from_rgb(color[0], color[1], color[2])
            }
            ColorType::Success => {
                let color = self.primary_color.success;
                egui::Color32::from_rgb(color[0], color[1], color[2])
            }
            ColorType::Warning => {
                let color = self.primary_color.warning;
                egui::Color32::from_rgb(color[0], color[1], color[2])
            }
            ColorType::Error => {
                let color = self.primary_color.error;
                egui::Color32::from_rgb(color[0], color[1], color[2])
            }
            ColorType::Info => {
                let color = self.primary_color.info;
                egui::Color32::from_rgb(color[0], color[1], color[2])
            }
        }
    }

    /// 保存主题到文件
    pub fn save_to_file(&self, path: &str) -> crate::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// 从文件加载主题
    pub fn load_from_file(path: &str) -> crate::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let theme: Self = serde_json::from_str(&json)?;
        Ok(theme)
    }

    /// 尝试从配置目录加载主题文件
    pub fn try_load_theme_from_config() -> Self {
        // 尝试从用户配置目录加载主题
        if let Some(mut config_dir) = dirs::config_dir() {
            config_dir.push("time_tracker");
            config_dir.push("theme.json");

            if config_dir.exists() {
                if let Ok(theme) = Self::load_from_file(&config_dir.to_string_lossy()) {
                    return theme;
                }
            }
        }

        // 如果加载失败，返回默认主题
        Self::default()
    }
}

/// 颜色类型
#[derive(Debug, Clone, Copy)]
pub enum ColorType {
    Primary,
    Secondary,
    Accent,
    Success,
    Warning,
    Error,
    Info,
}

/// 主题工具函数
pub mod theme_utils {
    use super::*;

    /// 创建带颜色的文本
    pub fn colored_text(text: &str, color_type: ColorType, theme: &Theme) -> egui::RichText {
        egui::RichText::new(text).color(theme.get_color(color_type))
    }

    /// 创建成功消息文本
    pub fn success_text(text: &str, theme: &Theme) -> egui::RichText {
        colored_text(text, ColorType::Success, theme)
    }

    /// 创建错误消息文本
    pub fn error_text(text: &str, theme: &Theme) -> egui::RichText {
        colored_text(text, ColorType::Error, theme)
    }

    /// 创建警告消息文本
    pub fn warning_text(text: &str, theme: &Theme) -> egui::RichText {
        colored_text(text, ColorType::Warning, theme)
    }

    /// 创建信息消息文本
    pub fn info_text(text: &str, theme: &Theme) -> egui::RichText {
        colored_text(text, ColorType::Info, theme)
    }

    /// 创建主题按钮
    pub fn themed_button(ui: &mut egui::Ui, text: &str, theme: &Theme) -> egui::Response {
        let button = egui::Button::new(text)
            .fill(theme.get_color(ColorType::Primary))
            .stroke(egui::Stroke::new(1.0, theme.get_color(ColorType::Accent)));
        ui.add(button)
    }

    /// 创建危险按钮
    pub fn danger_button(ui: &mut egui::Ui, text: &str, theme: &Theme) -> egui::Response {
        let button = egui::Button::new(text)
            .fill(theme.get_color(ColorType::Error))
            .stroke(egui::Stroke::new(1.0, theme.get_color(ColorType::Error)));
        ui.add(button)
    }

    /// 创建成功按钮
    pub fn success_button(ui: &mut egui::Ui, text: &str, theme: &Theme) -> egui::Response {
        let button = egui::Button::new(text)
            .fill(theme.get_color(ColorType::Success))
            .stroke(egui::Stroke::new(1.0, theme.get_color(ColorType::Success)));
        ui.add(button)
    }

    /// 获取状态颜色
    pub fn get_status_color(is_active: bool, theme: &Theme) -> egui::Color32 {
        if is_active {
            theme.get_color(ColorType::Success)
        } else {
            theme.get_color(ColorType::Error)
        }
    }

    /// 创建分隔线
    pub fn themed_separator(ui: &mut egui::Ui, theme: &Theme) {
        // 使用主题颜色创建自定义分隔线
        let separator_color = if theme.dark_mode {
            theme.get_color(ColorType::Secondary)
        } else {
            theme.get_color(ColorType::Primary)
        };

        // 使用主题颜色绘制分隔线
        ui.visuals_mut().widgets.noninteractive.bg_stroke.color = separator_color;
        ui.add(egui::Separator::default().spacing(theme.spacing.default));

        // 可以进一步自定义分隔线的外观
        // 这里使用了separator_color变量来设置分隔线颜色
    }

    /// 创建标题文本
    pub fn heading_text(text: &str, theme: &Theme) -> egui::RichText {
        egui::RichText::new(text)
            .size(theme.fonts.heading_size)
            .color(theme.get_color(ColorType::Primary))
            .strong()
    }

    /// 创建小标题文本
    pub fn subheading_text(text: &str, theme: &Theme) -> egui::RichText {
        egui::RichText::new(text)
            .size(theme.fonts.default_size * 1.2)
            .color(theme.get_color(ColorType::Secondary))
    }
}

impl ColorScheme {
    /// 获取红色分量
    pub fn r(&self) -> u8 {
        self.primary[0]
    }

    /// 获取绿色分量
    pub fn g(&self) -> u8 {
        self.primary[1]
    }

    /// 获取蓝色分量
    pub fn b(&self) -> u8 {
        self.primary[2]
    }

    /// 转换为egui::Color32
    pub fn to_color32(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.primary[0], self.primary[1], self.primary[2])
    }

    /// 从egui::Color32创建
    pub fn from_color32(color: egui::Color32) -> Self {
        Self {
            primary: [color.r(), color.g(), color.b()],
            ..Default::default()
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            primary: [70, 130, 180],
            secondary: [176, 196, 222],
            accent: [255, 140, 0],
            success: [34, 139, 34],
            warning: [255, 165, 0],
            error: [220, 20, 60],
            info: [30, 144, 255],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let theme = Theme::default();
        assert!(!theme.dark_mode);
        assert_eq!(theme.fonts.default_size, 14.0);
        assert_eq!(theme.spacing.default, 8.0);
    }

    #[test]
    fn test_theme_presets() {
        let light = Theme::from_preset(ThemePreset::Light);
        let dark = Theme::from_preset(ThemePreset::Dark);

        assert!(!light.dark_mode);
        assert!(dark.dark_mode);
    }

    #[test]
    fn test_color_adjustment() {
        let theme = Theme::default();
        let original = [100, 100, 100];
        let brightened = theme.brighten_color(original, 50);
        let darkened = theme.darken_color(original, 50);

        assert_eq!(brightened, [150, 150, 150]);
        assert_eq!(darkened, [50, 50, 50]);
    }

    #[test]
    fn test_toggle_dark_mode() {
        let mut theme = Theme::default();
        let original_mode = theme.dark_mode;

        theme.toggle_dark_mode();
        assert_eq!(theme.dark_mode, !original_mode);

        theme.toggle_dark_mode();
        assert_eq!(theme.dark_mode, original_mode);
    }

    #[test]
    fn test_color_type_conversion() {
        let theme = Theme::default();
        let primary_color = theme.get_color(ColorType::Primary);

        // 验证颜色转换正确
        assert_eq!(primary_color.r(), theme.primary_color.primary[0]);
        assert_eq!(primary_color.g(), theme.primary_color.primary[1]);
        assert_eq!(primary_color.b(), theme.primary_color.primary[2]);
    }
}
