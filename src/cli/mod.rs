//! # CLI模块
//!
//! 提供命令行界面功能，包括命令解析、执行和输出格式化

use crate::errors::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

pub mod commands;
pub mod output;

use output::*;

/// TimeTracker CLI应用程序
#[derive(Parser)]
#[command(name = "time_tracker")]
#[command(about = "一个强大的时间追踪工具", long_about = None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// 配置文件路径
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// 数据库文件路径
    #[arg(short, long, value_name = "FILE")]
    pub database: Option<PathBuf>,

    /// 启用详细输出
    #[arg(short, long)]
    pub verbose: bool,

    /// 静默模式（只输出错误）
    #[arg(short, long)]
    pub quiet: bool,

    /// 启动GUI模式
    #[arg(short, long)]
    pub gui: bool,

    /// 输出格式
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,

    /// 子命令
    #[command(subcommand)]
    pub command: Commands,
}

/// 支持的输出格式
#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum OutputFormat {
    /// 表格格式
    Table,
    /// JSON格式
    Json,
    /// CSV格式
    Csv,
    /// 简洁格式
    Simple,
}

/// CLI子命令
#[derive(Subcommand)]
pub enum Commands {
    /// 开始新任务
    Start {
        /// 任务名称
        task_name: String,
        /// 分类名称或ID
        #[arg(short, long)]
        category: Option<String>,
        /// 任务描述
        #[arg(short, long)]
        description: Option<String>,
        /// 标签（可多个）
        #[arg(short, long)]
        tags: Vec<String>,
    },

    /// 停止当前任务
    Stop {
        /// 任务描述（可选）
        #[arg(short, long)]
        description: Option<String>,
    },

    /// 暂停当前任务
    Pause,

    /// 恢复暂停的任务
    Resume,

    /// 显示当前状态
    Status,

    /// 列出时间记录
    List {
        /// 显示天数（默认7天）
        #[arg(short, long, default_value_t = 7)]
        days: u32,
        /// 分类过滤
        #[arg(short, long)]
        category: Option<String>,
        /// 标签过滤
        #[arg(short, long)]
        tag: Option<String>,
        /// 任务名称搜索
        #[arg(short, long)]
        search: Option<String>,
        /// 限制显示数量
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// 显示统计信息
    Stats {
        /// 统计周期
        #[arg(value_enum, default_value_t = StatsPeriod::ThisWeek)]
        period: StatsPeriod,
        /// 分类过滤
        #[arg(short, long)]
        category: Option<String>,
        /// 显示详细信息
        #[arg(short, long)]
        detailed: bool,
    },

    /// 分类管理
    Category {
        #[command(subcommand)]
        action: CategoryAction,
    },

    /// 配置管理
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// 数据导入导出
    Data {
        #[command(subcommand)]
        action: DataAction,
    },

    /// 报告生成
    Report {
        /// 报告类型
        #[arg(value_enum, default_value_t = ReportType::Daily)]
        report_type: ReportType,
        /// 开始日期 (YYYY-MM-DD)
        #[arg(short, long)]
        start_date: Option<String>,
        /// 结束日期 (YYYY-MM-DD)
        #[arg(short, long)]
        end_date: Option<String>,
        /// 输出文件路径
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// 统计周期
#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum StatsPeriod {
    /// 今天
    Today,
    /// 昨天
    Yesterday,
    /// 本周
    #[value(alias = "week")]
    ThisWeek,
    /// 上周
    LastWeek,
    /// 本月
    #[value(alias = "month")]
    ThisMonth,
    /// 上月
    LastMonth,
    /// 本年
    #[value(alias = "year")]
    ThisYear,
    /// 去年
    LastYear,
    /// 最近7天
    Last7Days,
    /// 最近30天
    Last30Days,
    /// 最近3个月
    Last3Months,
    /// 自定义
    Custom,
}

/// 分类操作
#[derive(Subcommand)]
pub enum CategoryAction {
    /// 列出所有分类
    List,
    /// 创建新分类
    Create {
        /// 分类名称
        name: String,
        /// 分类描述
        #[arg(short, long)]
        description: Option<String>,
        /// 分类颜色（十六进制）
        #[arg(short, long, default_value = "#2196F3")]
        color: String,
        /// 分类图标
        #[arg(short, long, default_value = "folder")]
        icon: String,
        /// 父分类
        #[arg(short, long)]
        parent: Option<String>,
    },
    /// 更新分类
    Update {
        /// 分类名称或ID
        category: String,
        /// 新名称
        #[arg(short, long)]
        name: Option<String>,
        /// 新描述
        #[arg(short, long)]
        description: Option<String>,
        /// 新颜色
        #[arg(short, long)]
        color: Option<String>,
        /// 新图标
        #[arg(short, long)]
        icon: Option<String>,
    },
    /// 删除分类
    Delete {
        /// 分类名称或ID
        category: String,
        /// 强制删除（不确认）
        #[arg(short, long)]
        force: bool,
    },
}

/// 配置操作
#[derive(Subcommand)]
pub enum ConfigAction {
    /// 显示当前配置
    Show,
    /// 设置配置项
    Set {
        /// 配置键
        key: String,
        /// 配置值
        value: String,
    },
    /// 获取配置项
    Get {
        /// 配置键
        key: String,
    },
    /// 重置配置
    Reset {
        /// 强制重置（不确认）
        #[arg(short, long)]
        force: bool,
    },
}

/// 数据操作
#[derive(Subcommand)]
pub enum DataAction {
    /// 导出数据
    Export {
        /// 输出文件路径
        output: PathBuf,
        /// 导出格式
        #[arg(value_enum, default_value_t = ExportFormat::Json)]
        format: ExportFormat,
        /// 开始日期
        #[arg(short, long)]
        start_date: Option<String>,
        /// 结束日期
        #[arg(short, long)]
        end_date: Option<String>,
    },
    /// 导入数据
    Import {
        /// 输入文件路径
        input: PathBuf,
        /// 导入格式
        #[arg(value_enum, default_value_t = ExportFormat::Json)]
        format: ExportFormat,
        /// 覆盖现有数据
        #[arg(short, long)]
        overwrite: bool,
    },
    /// 备份数据库
    Backup {
        /// 备份文件路径
        output: PathBuf,
    },
    /// 恢复数据库
    Restore {
        /// 备份文件路径
        input: PathBuf,
        /// 强制恢复（不确认）
        #[arg(short, long)]
        force: bool,
    },
}

/// 导出格式
#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum ExportFormat {
    /// JSON格式
    Json,
    /// CSV格式
    Csv,
    /// Excel格式
    Excel,
}

/// 报告类型
#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum ReportType {
    /// 每日报告
    Daily,
    /// 每周报告
    Weekly,
    /// 每月报告
    Monthly,
    /// 分类报告
    Category,
    /// 趋势报告
    Trend,
}

/// CLI应用程序
pub struct CliApp {
    /// CLI配置
    cli: Cli,
    /// 输出格式化器
    formatter: OutputFormatter,
}

impl CliApp {
    /// 创建新的CLI应用程序
    pub fn new(cli: Cli) -> Self {
        Self {
            formatter: OutputFormatter::new(cli.format.clone(), cli.verbose, cli.quiet),
            cli,
        }
    }

    /// 运行CLI应用程序
    pub async fn run(&self) -> Result<()> {
        // 执行命令
        match &self.cli.command {
            Commands::Start {
                task_name,
                category,
                description,
                tags,
            } => {
                self.handle_start(task_name, category.as_deref(), description.as_deref(), tags)
                    .await
            }
            Commands::Stop { description } => self.handle_stop(description.as_deref()).await,
            Commands::Pause => self.handle_pause().await,
            Commands::Resume => self.handle_resume().await,
            Commands::Status => self.handle_status().await,
            Commands::List {
                days,
                category,
                tag,
                search,
                limit,
            } => {
                self.handle_list(
                    *days,
                    category.as_deref(),
                    tag.as_deref(),
                    search.as_deref(),
                    *limit,
                )
                .await
            }
            Commands::Stats {
                period,
                category,
                detailed,
            } => {
                self.handle_stats(period, category.as_deref(), *detailed)
                    .await
            }
            Commands::Category { action } => self.handle_category(action).await,
            Commands::Config { action } => self.handle_config(action).await,
            Commands::Data { action } => self.handle_data(action).await,
            Commands::Report {
                report_type,
                start_date,
                end_date,
                output,
            } => {
                self.handle_report(
                    report_type,
                    start_date.as_deref(),
                    end_date.as_deref(),
                    output.as_ref().map(|p| p.as_path()),
                )
                .await
            }
        }
    }

    /// 显示成功消息
    fn show_success(&self, message: &str) {
        if !self.cli.quiet {
            println!("{} {}", "✓".green().bold(), message);
        }
    }

    /// 显示错误消息
    fn show_error(&self, message: &str) {
        eprintln!("{} {}", "✗".red().bold(), message.red());
    }

    /// 显示警告消息
    fn show_warning(&self, message: &str) {
        if !self.cli.quiet {
            println!("{} {}", "⚠".yellow().bold(), message.yellow());
        }
    }

    /// 显示信息消息
    fn show_info(&self, message: &str) {
        if !self.cli.quiet {
            println!("{} {}", "ℹ".blue().bold(), message);
        }
    }

    /// 确认操作
    fn confirm(&self, message: &str) -> Result<bool> {
        if self.cli.quiet {
            return Ok(false);
        }

        print!("{} {} [y/N]: ", "?".yellow().bold(), message);
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
    }
}

/// 解析CLI参数并创建应用程序
pub fn parse_cli() -> CliApp {
    let cli = Cli::parse();
    CliApp::new(cli)
}

/// 运行CLI应用程序
pub async fn run_cli() -> Result<()> {
    let app = parse_cli();
    app.run().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parsing() {
        // 测试CLI参数解析
        let cmd = Cli::command();
        cmd.debug_assert();
    }

    #[test]
    fn test_output_format_parsing() {
        use clap::Parser;

        let args = vec!["time_tracker", "--format", "json", "status"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.format, OutputFormat::Json);
    }

    #[test]
    fn test_start_command_parsing() {
        use clap::Parser;

        let args = vec![
            "time_tracker",
            "start",
            "测试任务",
            "--category",
            "工作",
            "--description",
            "这是一个测试任务",
            "--tags",
            "重要",
            "--tags",
            "紧急",
        ];

        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Start {
                task_name,
                category,
                description,
                tags,
            } => {
                assert_eq!(task_name, "测试任务");
                assert_eq!(category, Some("工作".to_string()));
                assert_eq!(description, Some("这是一个测试任务".to_string()));
                assert_eq!(tags, vec!["重要", "紧急"]);
            }
            _ => panic!("解析的命令类型不正确"),
        }
    }

    #[test]
    fn test_stats_command_parsing() {
        use clap::Parser;

        let args = vec![
            "time_tracker",
            "stats",
            "month",
            "--category",
            "工作",
            "--detailed",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Stats {
                period,
                category,
                detailed,
            } => {
                assert_eq!(period, StatsPeriod::ThisMonth);
                assert_eq!(category, Some("工作".to_string()));
                assert!(detailed);
            }
            _ => panic!("解析的命令类型不正确"),
        }
    }

    #[test]
    fn test_category_create_command() {
        use clap::Parser;

        let args = vec![
            "time_tracker",
            "category",
            "create",
            "新分类",
            "--description",
            "这是一个新分类",
            "--color",
            "#FF5722",
            "--icon",
            "work",
        ];

        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Category { action } => match action {
                CategoryAction::Create {
                    name,
                    description,
                    color,
                    icon,
                    parent,
                } => {
                    assert_eq!(name, "新分类");
                    assert_eq!(description, Some("这是一个新分类".to_string()));
                    assert_eq!(color, "#FF5722");
                    assert_eq!(icon, "work");
                    assert_eq!(parent, None);
                }
                _ => panic!("解析的分类操作不正确"),
            },
            _ => panic!("解析的命令类型不正确"),
        }
    }

    #[test]
    fn test_data_export_command() {
        use clap::Parser;
        use std::path::PathBuf;

        let args = vec![
            "time_tracker",
            "data",
            "export",
            "output.json",
            "--format",
            "json",
            "--start-date",
            "2024-01-01",
            "--end-date",
            "2024-12-31",
        ];

        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Data { action } => match action {
                DataAction::Export {
                    output,
                    format,
                    start_date,
                    end_date,
                } => {
                    assert_eq!(output, PathBuf::from("output.json"));
                    assert_eq!(format, ExportFormat::Json);
                    assert_eq!(start_date, Some("2024-01-01".to_string()));
                    assert_eq!(end_date, Some("2024-12-31".to_string()));
                }
                _ => panic!("解析的数据操作不正确"),
            },
            _ => panic!("解析的命令类型不正确"),
        }
    }
}
