# LifeTracker - 跨平台生活追踪应用

> 🚀 使用 Dioxus + Rust 构建的跨平台生活追踪应用

LifeTracker 是一个功能强大的综合生活追踪工具，帮助您管理生活的各个方面：时间追踪、财务记录、日记写作、习惯打卡等。让您的生活更有条理，提高效率。

## Development

Your new jumpstart project includes basic organization with an organized `assets` folder and a `components` folder.
If you chose to develop with the router feature, you will also have a `views` folder.

```
project/
├─ assets/ # Any assets that are used by the app should be placed here
├─ src/
│  ├─ main.rs # The entrypoint for the app. It also defines the routes for the app.
│  ├─ components/
│  │  ├─ mod.rs # Defines the components module
│  ├─ views/ # The views each route will render in the app.
│  │  ├─ mod.rs # Defines the module for the views route and re-exports the components for each route
├─ Cargo.toml # The Cargo.toml file defines the dependencies and feature flags for your project
```

### Tailwind
1. Install npm: https://docs.npmjs.com/downloading-and-installing-node-js-and-npm
2. Install the Tailwind CSS CLI: https://tailwindcss.com/docs/installation
3. Run the following command in the root of the project to start the Tailwind CSS compiler:

```bash
npx tailwindcss -i ./tailwind.css -o ./assets/tailwind.css --watch
```

### Serving Your App

Run the following command in the root of your project to start developing with the default platform:

```bash
dx serve
```

To run for a different platform, use the `--platform platform` flag. E.g.
```bash
dx serve --platform desktop
```

## ✨ 主要功能

### 🕒 时间追踪
- 精确的时间计算（基于系统时间戳）
- 任务分类和标签管理
- 暂停/恢复功能
- 实时状态同步

### 💰 财务管理
- 收入支出记录
- 分类统计
- 预算管理
- 财务报表

### 📝 日记功能
- 日常记录
- 心情追踪
- 富文本编辑
- 搜索和标签

### ✅ 习惯打卡
- 习惯追踪
- 连续打卡记录
- 进度可视化
- 目标设定

### 📊 数据统计
- 多维度分析
- 图表可视化
- 自定义报表
- 数据导出

## ⚙️ 系统管理
- 应用配置和主题设置
- 版本信息和关于页面
- 数据导入导出功能
- 系统信息查看

## 🛠️ 技术栈

- **UI框架**: Dioxus 0.6 + Rust (迁移中)
- **后端**: Rust + SQLite  
- **路由**: Dioxus Router
- **样式**: CSS/Tailwind（内联样式）
- **构建工具**: Dioxus CLI

> 📝 **迁移状态**: 从 Tauri + React 迁移到 Dioxus 进行中。时间追踪模块已完成，正准备开始财务管理模块。旧版React组件已归档到 `_tauri_archive/`。

## 🚀 快速开始

### 环境要求

- **Rust 1.75+** - 主要开发语言
- **CMake** - 构建系统依赖
- **Git** - 版本控制

### 环境搭建

#### 1. 安装 Rust

- Windows 环境
```powershell
winget install Rustlang.Rustup
```

- macOS 环境 (Homebrew 为例)
```bash
brew install rustup
```

- Linux 环境 (Debian 为例)
```bash
apt install rustup
```

#### 2. 安装 cargo-binstall
```powershell
cargo install cargo-binstall
```

#### 3. 安装 Dioxus CLI
```powershell
cargo binstall dioxus-cli
```

### 克隆和运行项目

```bash
# 克隆
git clone https://github.com/username/lifetracker-dioxus.git
cd lifetracker-dioxus

# 构建
cargo build

# 开发（桌面应用为例）
dx serve --platform desktop
```

## 📁 项目结构

```
life-tracker/
├── src/
│   ├── main.rs                 # 应用入口
│   ├── lib.rs                  # 库入口  
│   ├── components/             # UI组件 (模块化架构)
│   │   ├── mod.rs              # 模块声明
│   │   ├── app.rs              # 主应用组件 ✅
│   │   ├── dashboard.rs        # 主仪表板 ✅
│   │   ├── common.rs           # 通用组件 ✅
│   │   ├── timing/             # 时间追踪模块 ✅
│   │   │   ├── mod.rs          # 模块声明
│   │   │   ├── timing_page.rs  # 主页面入口（标签页导航）
│   │   │   ├── dashboard.rs    # 时间追踪仪表板
│   │   │   ├── task_management.rs # 任务管理
│   │   │   ├── category_management.rs # 分类管理
│   │   │   └── statistics.rs   # 统计报告
│   │   ├── accounting/         # 财务管理模块 ⏳
│   │   │   ├── mod.rs
│   │   │   ├── accounting_page.rs # 主页面入口
│   │   │   ├── overview.rs     # 财务概览
│   │   │   ├── accounts.rs     # 账户管理
│   │   │   ├── transactions.rs # 交易记录
│   │   │   ├── stats.rs        # 财务统计
│   │   │   └── trend_chart.rs  # 趋势图表
│   │   ├── diary/              # 日记模块 ⏳
│   │   │   ├── mod.rs
│   │   │   ├── diary_page.rs   # 主页面入口
│   │   │   ├── overview.rs     # 日记概览
│   │   │   ├── editor.rs       # 富文本编辑器
│   │   │   ├── library.rs      # 笔记库管理
│   │   │   └── stats.rs        # 写作统计
│   │   ├── settings/           # 设置模块 ✅
│   │   │   ├── mod.rs
│   │   │   ├── settings.rs     # 设置主页面
│   │   │   ├── about.rs        # 关于页面
│   │   │   └── system_page.rs  # 系统页面入口
│   │   ├── data_management/    # 数据管理子模块 ✅
│   │   │   ├── mod.rs
│   │   │   ├── data_management_page.rs # 数据管理主页面
│   │   │   ├── export.rs       # 数据导出
│   │   │   ├── import.rs       # 数据导入
│   │   │   ├── backup.rs       # 数据备份 ⏳
│   │   │   ├── sync.rs         # 数据同步 ⏳
│   │   │   └── cleanup.rs      # 数据清理 ⏳
│   │   ├── habits/             # 习惯打卡模块 ⏳
│   │   │   ├── mod.rs
│   │   │   └── habits_page.rs  # 习惯打卡页面
│   │   ├── Timing/             # [待迁移] React时间追踪组件
│   │   ├── Accounting/         # [待迁移] React财务组件  
│   │   ├── Notes/              # [待迁移] React日记组件
│   │   ├── DataManagement/     # [待迁移] React数据管理组件
│   │   └── Animation/          # [待迁移] React动画组件
│   ├── storage/                # 数据存储层
│   │   ├── database.rs         # 数据库操作
│   │   ├── models.rs           # 数据模型
│   │   └── migrations.rs       # 数据库迁移
│   ├── core/                   # 核心业务逻辑
│   │   ├── timer.rs            # 计时器逻辑
│   │   ├── category.rs         # 分类管理
│   │   ├── task.rs             # 任务管理
│   │   └── analytics.rs        # 数据分析
│   ├── utils/                  # 工具函数
│   │   ├── format.rs           # 格式化工具
│   │   └── validation.rs       # 验证工具
│   └── errors.rs               # 错误处理
├── _tauri_archive/             # 旧版React组件归档
├── assets/                     # 静态资源
├── Cargo.toml                  # 项目配置
└── TAURI_TO_DIOXUS_REFERENCE.md # 迁移参考文档
```

**图例**: ✅ 已迁移 | 🔄 进行中 | ⏳ 待迁移

**迁移进度**: 时间追踪模块和系统管理模块已完成，财务管理模块为下一目标

## 💾 数据存储

应用数据存储在以下位置：

```
LifeTracker/
├── config.toml             # 应用配置
├── lifetracker.db         # SQLite 数据库
└── logs/                   # 应用日志
```

**数据目录位置**：
- **Windows**: `%APPDATA%\LifeTracker\`
- **macOS**: `~/Library/Application Support/lifetracker/`
- **Linux**: `~/.local/share/lifetracker/`

## 🎨 主题支持

- 🌞 浅色主题
- 🌙 深色主题
- 🎯 自动跟随系统

## 📊 数据导入导出

支持多种格式的数据导入导出：

- CSV 格式
- JSON 格式
- XML 格式
- Markdown 报告

## 🔧 开发指南

### 开发命令

```bash
# 开发模式（桌面应用）
dx serve

# 开发模式（Web版本）
dx serve --platform web

# 构建生产版本（桌面）
dx build --platform desktop --release

# 构建生产版本（Web）
dx build --platform web --release

# 代码格式化
cargo fmt

# 代码检查
cargo clippy

# 运行测试
cargo test
```

### 代码规范

- 使用 `cargo fmt` 进行代码格式化
- 使用 `cargo clippy` 进行代码检查
- 遵循 Rust 官方编码规范
- 注释使用中文，日志使用英文

### Dioxus 组件开发

```rust
use dioxus::prelude::*;

#[component]
fn TimingPage() -> Element {
    let active_tab = use_state(|| "dashboard");
    
    rsx! {
        div { class: "flex flex-col h-full",
            // 标签导航
            div { class: "flex border-b border-gray-200 dark:border-gray-700",
                button {
                    class: if *active_tab.read() == "dashboard" { 
                        "px-4 py-2 text-theme-primary border-b-2 border-theme-primary" 
                    } else { 
                        "px-4 py-2 text-gray-500 hover:text-gray-700" 
                    },
                    onclick: move |_| active_tab.set("dashboard"),
                    "仪表板"
                }
                // 其他标签...
            }
            
            // 内容区域
            div { class: "flex-1 p-4",
                match active_tab.read().as_str() {
                    "dashboard" => rsx! { DashboardTab {} },
                    "tasks" => rsx! { TaskManagementTab {} },
                    _ => rsx! { div { "未知页面" } }
                }
            }
        }
    }
}
```

### 提交规范

```bash
# 功能开发
feat: 添加财务记录功能

# 问题修复
fix: 修复计时器暂停后无法继续的问题

# 性能优化
perf: 优化任务列表渲染性能

# 重构
refactor: 重构数据库查询逻辑
```

## 🚀 部署

### 桌面应用

```bash
# 构建桌面应用
dx build --platform desktop --release

# 输出位置
# Windows: target/dx/lifetracker-dioxus/release/bundle/msi/
# macOS: target/dx/lifetracker-dioxus/release/bundle/dmg/
# Linux: target/dx/lifetracker-dioxus/release/bundle/appimage/
```

### Web 应用

```bash
# 构建 Web 应用
dx build --platform web --release

# 输出位置: dist/
# 可以部署到任何静态文件服务器
```

## 🔄 从 Tauri 迁移

如果你有现有的 Tauri 版本，可以按照以下步骤迁移：

1. **保留数据库** - SQLite 数据库可以直接复用
2. **迁移 Rust 代码** - 核心业务逻辑无需修改
3. **重写 UI 组件** - 从 React 组件改为 Dioxus 组件
4. **更新构建配置** - 使用 Dioxus CLI 替代 Tauri CLI

详细迁移指南请参考：[dioxus-migration-guide.md](./dioxus-migration-guide.md)

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

### 开发流程

1. Fork 项目
2. 创建特性分支
3. 提交更改
4. 推送到分支
5. 创建 Pull Request

## 📄 许可证

MIT License

## 📞 联系方式

- 📧 Email: contact@lifetracker.dev
- 🌐 Website: https://lifetracker.dev
- 📱 GitHub: https://github.com/lifetracker/lifetracker-dioxus

---

**LifeTracker** - 让生活更有条理 ✨