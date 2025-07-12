# Tauri 到 Dioxus 迁移参考文档

## 📋 概述

本文档记录了从 Tauri + React 迁移到 Dioxus 过程中需要保持一致性的组件行为和功能。
所有旧版本的代码已归档到 `_tauri_archive/` 目录，迁移时可参考。

## 🏗️ 架构对比

### Tauri 版本架构
```
前端 (React + TypeScript)
    ↓ invoke() 调用
后端 (Rust + Tauri Commands)
    ↓ 数据库操作
SQLite 数据库
```

### Dioxus 版本架构
```
Dioxus UI (Rust)
    ↓ 直接函数调用
核心业务逻辑 (Rust)
    ↓ 数据库操作
SQLite 数据库
```

## 📱 组件迁移对照表

### 四大主页面

| 页面模块 | Tauri React 入口 | Dioxus 模块 | 状态 | 核心功能 | 子模块数量 |
|----------|------------------|-------------|------|----------|----------|
| **时间追踪** | `Timing/TimingPage.tsx` | `components/timing/` | ✅ 已完成 | 任务管理、计时器、分类、统计 | 5个rs |
| **财务管理** | `Accounting/AccountingPage.tsx` | `components/accounting/` | ✅ 已完成 | 账户、交易、预算、报表 | 6个rs |
| **日记笔记** | `NotesPage.tsx` | `components/diary/` | ✅ 已完成 | 编辑器、库管理、统计 | 5个rs |
| **系统管理** | `SystemPage.tsx` | `components/settings/` | ⏳ 待迁移 | 数据管理、设置、关于 | 11个rs |

### 子模块详细对照

#### 1. 时间追踪模块 (timing/)
| React 组件 | Dioxus 模块 | 核心功能 | 迁移状态 | 优先级 |
|------------|-------------|----------|----------|--------|
| `TimingPage.tsx` | `timing_page.rs` | 标签页导航、路由 | ✅ 已完成 | 🔴 高 |
| `Dashboard.tsx` | `dashboard.rs` | 实时计时器、今日统计 | ✅ 已完成 | 🔴 高 |
| `TaskManagement.tsx` | `task_management.rs` | 任务CRUD、列表管理 | ✅ 已完成 | 🔴 高 |
| `CategoryManagement.tsx` | `category_management.rs` | 分类管理、颜色图标 | ✅ 已完成 | 🟡 中 |
| `Statistics.tsx` | `statistics.rs` | 时间统计、图表展示 | ✅ 已完成 | 🟡 中 |

#### 2. 财务管理模块 (accounting/)
| React 组件 | Dioxus 模块 | 核心功能 | 迁移状态 | 优先级 |
|------------|-------------|----------|----------|--------|
| `AccountingPage.tsx` | `accounting_page.rs` | 标签页导航、路由 | ✅ 已完成 | 🟡 中 |
| `OverviewTab.tsx` | `overview.rs` | 资产概览、余额汇总 | ✅ 已完成 | 🟡 中 |
| `AccountsTab.tsx` | `accounts.rs` | 账户管理 | ✅ 已完成 | 🟡 中 |
| `TransactionsTab.tsx` | `transactions.rs` | 交易记录、CRUD | ✅ 已完成 | 🟡 中 |
| `StatsTab.tsx` | `stats.rs` | 财务统计分析 | ✅ 已完成 | 🟢 低 |
| `FinancialTrendChart.tsx` | `trend_chart.rs` | 趋势图表 | ✅ 已完成 | 🟢 低 |

#### 3. 日记笔记模块 (diary/)
| React 组件 | Dioxus 模块 | 核心功能 | 迁移状态 | 优先级 |
|------------|-------------|----------|----------|--------|
| `NotesPage.tsx` | `diary_page.rs` | 标签页导航、路由 | ✅ 已完成 | 🟢 低 |
| `NotesOverview.tsx` | `overview.rs` | 笔记概览、快速访问 | ✅ 已完成 | 🟢 低 |
| `NotesEditor.tsx` | `editor.rs` | 富文本编辑器 | ✅ 已完成 | 🟢 低 |
| `NotesLibrary.tsx` | `library.rs` | 笔记库管理、搜索 | ✅ 已完成 | 🟢 低 |
| `NotesStats.tsx` | `stats.rs` | 写作统计 | ✅ 已完成 | 🟢 低 |

#### 4. 系统管理模块 (settings/)
| React 组件 | Dioxus 模块 | 核心功能 | 迁移状态 | 优先级 |
|------------|-------------|----------|----------|--------|
| `SystemPage.tsx` | `system_page.rs` | 系统页面导航、路由 | ✅ 已完成 | 🟡 中 |
| `Settings.tsx` | `settings.rs` | 应用设置、主题配置 | ✅ 已完成 | 🟡 中 |
| `About.tsx` | `about.rs` | 版本信息、许可证 | ✅ 已完成 | 🟢 低 |
| `DataManagement.tsx` | `data_management/data_management_page.rs` | 数据管理入口 | ✅ 已完成 | 🟡 中 |
| `DataExport.tsx` | `data_management/export.rs` | 数据导出功能 | ✅ 已完成 | 🟢 低 |
| `DataImport.tsx` | `data_management/import.rs` | 数据导入功能 | ✅ 已完成 | 🟢 低 |
| `DataBackup.tsx` | `data_management/backup.rs` | 备份恢复 | ⏳ 待迁移 | 🟢 低 |
| `DataSync.tsx` | `data_management/sync.rs` | 同步功能 | ⏳ 待迁移 | 🟢 低 |
| `DataCleanup.tsx` | `data_management/cleanup.rs` | 数据清理 | ⏳ 待迁移 | 🟢 低 |
| `ConflictResolution.tsx` | `data_management/conflict_resolution.rs` | 冲突解决 | ⏳ 待迁移 | 🟢 低 |

### 通用组件模块
| React 组件 | Dioxus 模块 | 核心功能 | 迁移状态 | 优先级 |
|------------|-------------|----------|----------|--------|
| `Animation/` (7个) | `animation.rs` | 页面过渡、动画效果 | ✅ 已完成 | 🟡 中 |
| `ErrorBoundary.tsx` | `common.rs` | 错误边界处理 | ✅ 已完成 | 🟡 中 |
| `TitleBar.tsx` | `title_bar.rs` | 自定义标题栏 | ✅ 已完成 | 🟢 低 |

### 习惯打卡模块
| React 组件 | Dioxus 模块 | 核心功能 | 迁移状态 | 优先级 |
|------------|-------------|----------|----------|--------|
| (待设计) | `habits/habits_page.rs` | 习惯追踪、打卡记录 | ⏳ 待迁移 | 🟢 低 |

## 🔧 核心功能行为参考

### 1. 任务管理 (TaskManagement)

#### 原始行为 (参考 `_tauri_archive/components/TaskManagement.tsx`)
```typescript
// 关键行为：
1. 实时计时器显示 (每秒更新)
2. 开始任务时自动创建时间记录
3. 暂停/恢复功能保持状态
4. 停止时计算总时长并保存
5. 任务列表支持分类过滤
6. 支持任务描述和标签添加
```

#### Dioxus 实现要点
```rust
// 需要保持的核心行为：
- use_interval: 实现每秒更新的计时器
- use_state: 管理当前任务状态
- use_effect: 监听任务状态变化并同步数据库
- 错误处理: 网络断开时的本地缓存机制
```

### 2. 分类管理 (CategoryManagement)

#### 原始行为
```typescript
// 关键功能：
1. 颜色选择器 - 预设颜色 + 自定义颜色
2. 图标选择器 - 预设图标库
3. 拖拽排序 - 修改分类显示顺序
4. 嵌套分类 - 支持父子关系
5. 目标设置 - 每日/每周时间目标
```

#### Dioxus 实现要点
```rust
// 重要保持项：
- 颜色选择器的颜色值格式保持一致
- 图标字符串格式不变 (emoji 或 icon class)
- 排序算法保持一致
- 表单验证规则相同
```

### 3. 数据统计 (Statistics)

#### 原始行为
```typescript
// 核心功能：
1. 时间范围选择器 (日/周/月/自定义)
2. 图表类型切换 (柱状图/饼图/折线图)
3. 分类筛选和对比
4. 数据导出功能
5. 实时数据更新
```

#### Dioxus 实现要点
```rust
// 替换方案：
- Chart.js -> plotters-rs 或 egui_plot
- 日期选择器 -> 自定义日期组件
- 数据格式保持一致以支持导出
```

## 🔄 数据交互模式

### Tauri 版本
```typescript
// React 调用模式
const result = await invoke<TaskList>('get_tasks', { 
  category_id: selectedCategory 
});
```

### Dioxus 版本
```rust
// 直接调用模式
let storage = use_context::<AppState>().get_storage();
let tasks = storage.get_tasks(category_id).await?;
```

## 🎨 UI/UX 一致性要求

### 1. 主题系统
- 浅色/深色主题切换保持一致
- CSS 变量名称保持相同
- 颜色值精确匹配

### 2. 响应式设计
- 断点保持相同 (sm: 640px, md: 768px, lg: 1024px)
- 网格系统布局一致
- 移动端交互行为相同

### 3. 动画效果
- 页面切换动画保持
- 按钮 hover 效果一致
- 加载状态动画相同

## 📊 状态管理对比

### Tauri 版本
```typescript
// React State + Context
const [tasks, setTasks] = useState<Task[]>([]);
const { config, updateConfig } = useConfig();
```

### Dioxus 版本
```rust
// Dioxus Hooks + Context
let tasks = use_state(|| Vec::<Task>::new());
let config = use_context::<AppState>();
```

## 🚀 迁移检查清单

### 功能完整性
- [ ] 所有 CRUD 操作正常
- [ ] 计时器精度保持一致 (±1秒)
- [ ] 数据导入导出格式相同
- [ ] 快捷键支持保持
- [ ] 系统通知功能
- [ ] 主题切换流畅

### 性能要求
- [ ] 启动时间 < 3秒
- [ ] 大数据量 (>1000条记录) 流畅操作
- [ ] 内存使用 < 200MB
- [ ] 数据库查询响应 < 100ms

### 用户体验
- [ ] 界面布局像素级一致
- [ ] 交互流程完全相同
- [ ] 错误提示信息一致
- [ ] 加载状态显示一致

## 📝 迁移日志

### 已完成 ✅
- ✅ 2025-01-11: 创建基础 Dioxus 项目结构
- ✅ 2025-01-11: 迁移主应用入口 (`app.rs`)
- ✅ 2025-01-11: 迁移主仪表板 (`dashboard.rs`)
- ✅ 2025-01-11: 创建通用组件模块 (`common.rs`)
- ✅ 2025-01-11: 迁移核心数据库逻辑
- ✅ 2025-01-11: 完成时间追踪页面 (`timing.rs`)
  - ✅ 主计时器和任务管理界面
  - ✅ 分类管理功能 (列表展示、搜索、图标渲染)
  - ✅ 统计报告页面框架
  - ✅ 四标签页导航结构
- ✅ 2025-01-11: 完成系统管理模块 (`settings/`)
  - ✅ 系统设置页面 (`settings.rs`) - 563行，6个设置分类
  - ✅ 关于页面 (`about.rs`) - 477行，完整应用信息
  - ✅ 系统主页面 (`system_page.rs`) - 234行，导航入口
  - ✅ 数据管理模块 (`data_management/`)
    - ✅ 数据管理主页面 (`data_management_page.rs`) - 实时统计和导航
    - ✅ 数据导出功能 (`export.rs`) - 5种格式，完整选项
    - ✅ 数据导入功能 (`import.rs`) - 多格式支持，安全确认
- ✅ 2025-01-11: 完成财务管理模块 (`accounting/`)
  - ✅ 财务管理主页面 (`accounting_page.rs`) - 249行，完整标签页导航
  - ✅ 财务概览页面 (`overview.rs`) - 196行，统计卡片和最近交易
  - ✅ 账户管理页面 (`accounts.rs`) - 118行，网格布局和空状态
  - ✅ 交易记录页面 (`transactions.rs`) - 235行，表格和卡片双布局
  - ✅ 财务统计页面 (`stats.rs`) - 449行，包含趋势图表控制
  - ✅ 趋势图表组件 (`trend_chart.rs`) - 192行，完整柱状图组件
  - ✅ 修复主应用路由连接
- ✅ 2025-01-11: 完成日记笔记模块 (`diary/`)
  - ✅ 日记主页面 (`diary_page.rs`) - 86行，标签页导航和路由
  - ✅ 日记概览页面 (`overview.rs`) - 117行，统计卡片和空状态
  - ✅ 日记编辑器页面 (`editor.rs`) - 243行，心情选择和标签管理
  - ✅ 日记库页面 (`library.rs`) - 261行，搜索过滤和视图切换
  - ✅ 日记统计页面 (`stats.rs`) - 384行，标签分布和心情统计
  - ✅ 修复主应用路由连接
- ✅ 2025-01-11: 完成通用组件模块迁移
  - ✅ 基础UI组件库 (`common.rs`, 753行) - 按钮、卡片、输入框、空状态等10个组件
  - ✅ 错误边界组件 (`common.rs`) - 错误处理、错误显示、状态管理
  - ✅ 标题栏组件 (`title_bar.rs`, 120行) - 窗口控制、拖拽区域、移动端适配
  - ✅ 动画组件库 (`animation.rs`, 420行) - 7个动画组件，CSS过渡，状态管理
    - ✅ PageTransition - 页面过渡动画
    - ✅ ViewContainer - 视图容器动画
    - ✅ InteractiveButton - 交互式按钮
    - ✅ TabTransition - 标签页过渡
    - ✅ AnimatedList - 动画列表
    - ✅ BottomSheet - 底部抽屉
    - ✅ GestureWrapper - 手势包装器

### 进行中 🔄
- 无正在进行的迁移任务

### 计划中 ⏳
**按优先级排序**：

1. **系统管理模块剩余功能** (🟢 低优先级)
   - ⏳ 数据备份功能 (`data_management/backup.rs`)
   - ⏳ 数据同步功能 (`data_management/sync.rs`)
   - ⏳ 数据清理功能 (`data_management/cleanup.rs`)
   - ⏳ 冲突解决功能 (`data_management/conflict_resolution.rs`)

## 🔍 测试验证

### 自动化测试
```bash
# Tauri 版本测试
pnpm test

# Dioxus 版本测试
cargo test
```

### 手动测试检查项
1. **功能对等性**: 每个功能在两个版本中行为完全一致
2. **数据一致性**: 同一数据库在两个版本中显示相同
3. **性能对比**: 关键操作时间不超过旧版本的 120%
4. **错误处理**: 异常情况处理方式保持一致

## 📚 参考资源

### 归档代码位置
- React 组件: `_tauri_archive/components/`
- React Hooks: `_tauri_archive/hooks/`
- Context 提供者: `_tauri_archive/contexts/`
- TypeScript 类型: `_tauri_archive/types/`
- Tauri 命令: `_tauri_archive/tauri_commands/`

### 迁移工具
- Dioxus 官方文档: https://dioxuslabs.com/
- React to Dioxus 对比: https://dioxuslabs.com/learn/0.5/migration/react
- Rust 异步编程: https://tokio.rs/

---

**⚠️ 重要提醒**: 在删除任何归档文件之前，确保对应的 Dioxus 组件已完全实现并通过测试验证！ 