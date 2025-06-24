# TimeTracker - 时间追踪应用程序

一个功能完整的时间追踪应用程序，基于 Tauri + React + Rust 开发，提供现代化的桌面应用体验。

## 功能特性

### 核心功能
- ⏱️ **时间记录**: 精确记录任务时间，支持开始/暂停/停止操作
- 📊 **统计分析**: 多维度数据统计和效率评分
- 🏷️ **分类管理**: 灵活的任务分类系统
- 📝 **任务管理**: 完整的任务生命周期管理
- 🔍 **数据查询**: 强大的搜索和过滤功能

### 用户界面
- 🖥️ **现代化界面**: 基于 React + Tailwind CSS 的响应式界面
- 🌙 **深色模式**: 支持浅色/深色主题切换
- 📱 **响应式设计**: 适配不同屏幕尺寸
- 🎛️ **可拖拽侧栏**: 桌面端优化的交互体验

### 数据管理
- 💾 **本地存储**: 基于SQLite的可靠数据存储
- 📤 **数据导出**: 支持JSON、CSV、XML、HTML、Markdown格式
- 📥 **数据导入**: 支持多种格式的数据导入
- 🔄 **实时同步**: 前后端数据实时同步

## 技术栈

- **前端**: React + TypeScript + Tailwind CSS
- **后端**: Rust + Tauri
- **数据库**: SQLite
- **构建工具**: Cargo + Vite + pnpm

## 开发环境设置

### 系统要求
- Node.js 16+ 和 pnpm
- Rust 1.70+
- 操作系统: Windows 10/11, macOS 10.15+, 或 Linux

### 安装依赖

```bash
# 克隆仓库
git clone https://github.com/username/time-tracker.git
cd time-tracker

# 安装前端依赖
pnpm install

# 确保Rust环境就绪
cargo --version
```

## 开发和构建

### 开发模式
```bash
# 启动开发服务器（热重载）
pnpm tauri dev
```

### 构建发布版本
```bash
# 构建生产版本
pnpm tauri build
```

### 其他命令
```bash
# 清理构建文件
pnpm tauri clean

# 运行Rust测试
cargo test

# 代码格式化
cargo fmt

# 代码质量检查
cargo clippy
```

## 文件存储

### 开发环境
在开发环境中（使用 `cargo run`），所有配置和数据文件都存储在项目根目录的 `data/` 文件夹中：

```
time-tracker/
├── data/
│   ├── config/
│   │   ├── theme.json      # 主题配置
│   │   └── config.toml     # 应用配置
│   ├── logs/               # 日志文件
│   ├── backups/            # 数据库备份
│   ├── exports/            # 导出数据
│   └── timetracker.db      # SQLite 数据库
└── (其他项目文件)
```

### 生产环境
在生产环境中，文件存储在操作系统的标准位置：

- **Windows**: `%APPDATA%\TimeTracker\`
- **macOS**: `~/Library/Application Support/timetracker/`  
- **Linux**: `~/.local/share/timetracker/`

### 配置文件管理
- 主题配置会自动保存和加载
- 设置页面显示实际的配置文件路径
- 开发时所有文件都在项目内，不会污染系统目录

## 使用指南

启动应用程序后：

1. **创建任务**: 在仪表板点击"快速开始"创建新任务
2. **开始计时**: 选择任务后点击"开始"按钮
3. **时间管理**: 使用暂停/停止控制计时状态
4. **查看统计**: 在统计报告页面查看详细分析
5. **分类管理**: 在分类管理页面组织任务分类

## 项目结构

```
time-tracker/
├── src/                   # 前端源码 (React + TypeScript)
│   ├── App.tsx           # 主应用组件
│   ├── components/       # React组件
│   ├── hooks/            # 自定义Hooks
│   ├── types/            # TypeScript类型定义
│   └── main.tsx          # 入口文件
├── src/                  # Rust后端源码 (与前端在同一src目录)
│   ├── main.rs          # Tauri程序入口
│   ├── lib.rs           # 库文件
│   ├── core/            # 核心业务逻辑
│   ├── storage/         # 数据存储层
│   ├── config/          # 配置管理
│   ├── utils/           # 工具函数
│   └── tauri_commands.rs # Tauri命令
├── package.json         # 前端依赖
├── Cargo.toml          # Rust依赖
├── tauri.conf.json     # Tauri配置
└── vite.config.ts      # Vite配置
```

## 核心特性

### 效率评分系统
- **专注度评分** (40分): 基于平均会话时长
- **工作量评分** (30分): 基于总工作时长  
- **节奏评分** (30分): 基于工作段数与时长平衡

### 响应式设计
- 手机端: 1列卡片布局
- 平板端: 2列卡片布局
- 桌面端: 4列卡片布局
- 可拖拽侧栏: 支持宽度调整和折叠

### 实时数据同步
- 计时器状态实时更新
- 统计数据自动刷新
- 今日工作记录即时显示

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🎯 开发路线图

- [x] 基础项目结构
- [ ] 核心计时功能
- [ ] 数据库集成
- [ ] 统计分析
- [ ] 系统集成
- [ ] 数据导出
- [ ] 性能优化

---

**开始你的时间管理之旅！** ⏰