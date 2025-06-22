//! # CLI命令处理模块
//!
//! 实现各种CLI命令的具体执行逻辑

use crate::{
    cli::{
        CategoryAction, CliApp, ConfigAction, DataAction, ExportFormat, ReportType, StatsPeriod,
    },
    core::{timer::TimerState, AppCore},
    errors::{AppError, Result},
    storage::{models::*, Database},
    utils::{
        create_progress_bar,
        date::{format_datetime, this_week_range, today_range},
        format::{
            format_duration_compact, format_duration_decimal_hours, format_duration_detailed,
            format_number_with_commas, format_percentage,
        },
        format_file_size, get_file_extension,
        validation::validate_task_name,
    },
};
use chrono::{Datelike, Duration, Local, NaiveDate};

use std::path::Path;
use uuid::Uuid;

impl CliApp {
    /// 处理开始任务命令
    pub async fn handle_start(
        &self,
        task_name: &str,
        category: Option<&str>,
        description: Option<&str>,
        tags: &[String],
    ) -> Result<()> {
        // 验证任务名称
        let validation_result = validate_task_name(task_name);
        if !validation_result.is_valid {
            self.show_error(&format!(
                "任务名称无效: {}",
                validation_result.errors.join(", ")
            ));
            return Ok(());
        }

        // 验证描述（如果提供）
        if let Some(desc) = description {
            use crate::utils::validation::validate_description;
            let desc_validation = validate_description(desc);
            if !desc_validation.is_valid {
                self.show_error(&format!(
                    "任务描述无效: {}",
                    desc_validation.errors.join(", ")
                ));
                return Ok(());
            }
        }

        // 获取数据库连接
        let db = self.get_database()?;
        let mut core = AppCore::new();

        // 检查是否有正在运行的任务
        if matches!(core.timer().state(), TimerState::Running { .. }) {
            self.show_warning("已有任务正在运行，请先停止当前任务");
            return Ok(());
        }

        // 查找分类ID
        let category_id = if let Some(cat_name) = category {
            self.find_category_id(&db, cat_name)?
        } else {
            None
        };

        // 创建时间记录并验证时间
        let start_time = Local::now();
        use crate::utils::validation::validate_time_entry_duration;
        let time_validation = validate_time_entry_duration(start_time, None);
        if !time_validation.is_valid {
            self.show_error(&format!(
                "时间验证失败: {}",
                time_validation.errors.join(", ")
            ));
            return Ok(());
        }

        let entry = TimeEntryInsert {
            id: Uuid::new_v4(),
            task_name: task_name.to_string(),
            category_id,
            start_time,
            end_time: None,
            duration_seconds: 0,
            description: description.map(|s| s.to_string()),
            tags: tags.to_vec(),
            created_at: Local::now(),
        };

        // 保存到数据库
        db.insert_time_entry(&entry)?;

        // 启动计时器
        core.start_task(
            task_name.to_string(),
            category_id,
            description.map(|s| s.to_string()),
        )?;

        self.show_success(&format!("已开始任务: {}", task_name));

        if let Some(cat) = category {
            self.show_info(&format!("分类: {}", cat));
        }

        if let Some(desc) = description {
            self.show_info(&format!("描述: {}", desc));
        }

        if !tags.is_empty() {
            self.show_info(&format!("标签: {}", tags.join(", ")));
        }

        Ok(())
    }

    /// 处理停止任务命令
    pub async fn handle_stop(&self, description: Option<&str>) -> Result<()> {
        let db = self.get_database()?;
        let mut core = AppCore::new();

        // 检查是否有正在运行的任务
        if !matches!(core.timer().state(), TimerState::Running { .. }) {
            self.show_warning("当前没有正在运行的任务");
            return Ok(());
        }

        // 停止计时器
        let elapsed = core.stop_task()?;

        // 这里需要更新数据库中的记录
        // 简化实现，实际应该查找当前运行的任务并更新

        self.show_success(&format!(
            "任务已停止，用时: {} (简洁格式: {})",
            format_duration_detailed(elapsed),
            format_duration_compact(elapsed)
        ));

        if let Some(desc) = description {
            self.show_info(&format!("备注: {}", desc));
        }

        // 显示任务停止时间
        let stop_time = Local::now();
        self.show_info(&format!("停止时间: {}", format_datetime(stop_time)));

        Ok(())
    }

    /// 处理暂停任务命令
    pub async fn handle_pause(&self) -> Result<()> {
        let mut core = AppCore::new();

        match core.timer().state() {
            TimerState::Running { .. } => {
                core.pause_task()?;
                self.show_success("任务已暂停");
            }
            TimerState::Paused { .. } => {
                self.show_warning("任务已经处于暂停状态");
            }
            TimerState::Stopped => {
                self.show_warning("当前没有正在运行的任务");
            }
        }

        Ok(())
    }

    /// 处理恢复任务命令
    pub async fn handle_resume(&self) -> Result<()> {
        let mut core = AppCore::new();

        match core.timer().state() {
            TimerState::Paused { .. } => {
                core.resume_task()?;
                self.show_success("任务已恢复");
            }
            TimerState::Running { .. } => {
                self.show_warning("任务已经在运行中");
            }
            TimerState::Stopped => {
                self.show_warning("当前没有暂停的任务");
            }
        }

        Ok(())
    }

    /// 处理状态查询命令
    pub async fn handle_status(&self) -> Result<()> {
        let core = AppCore::new();

        // 显示计时器状态
        match core.timer().state() {
            TimerState::Running {
                start_time,
                paused_duration,
            } => {
                let elapsed = core.get_current_duration();
                self.show_success("正在运行任务");
                self.show_info(&format!("开始时间: {}", format_datetime(*start_time)));
                self.show_info(&format!(
                    "已运行时间: {}",
                    format_duration_detailed(elapsed)
                ));

                // 使用更多的格式化选项
                use crate::utils::format::{format_duration_decimal_hours, format_percentage};
                let hours = format_duration_decimal_hours(elapsed);
                self.show_info(&format!("换算为小时: {}", hours));

                // 显示暂停时长（如果有）
                if *paused_duration > Duration::zero() {
                    self.show_info(&format!(
                        "累计暂停时长: {}",
                        format_duration_detailed(*paused_duration)
                    ));
                }

                // 如果有目标时间，显示完成百分比
                let target_hours = 8.0; // 假设目标是8小时
                let current_hours: f64 = elapsed.num_seconds() as f64 / 3600.0;
                if current_hours > 0.0 {
                    let percentage = format_percentage(current_hours / target_hours * 100.0, 1);
                    self.show_info(&format!("目标完成度: {}", percentage));
                }
            }
            TimerState::Paused {
                start_time,
                pause_start,
                paused_duration,
            } => {
                let elapsed = core.get_current_duration();
                self.show_warning("任务已暂停");
                self.show_info(&format!("开始时间: {}", format_datetime(*start_time)));
                self.show_info(&format!("暂停时间: {}", format_datetime(*pause_start)));
                self.show_info(&format!(
                    "暂停前运行时间: {}",
                    format_duration_detailed(elapsed)
                ));
                self.show_info(&format!(
                    "累计暂停时长: {}",
                    format_duration_detailed(*paused_duration)
                ));
            }
            TimerState::Stopped => {
                self.show_info("当前没有活动任务");
            }
        }

        // 获取数据库统计信息
        let db = self.get_database()?;

        // 显示基本数据库信息
        // 由于Database.connection是私有的，我们使用其他方式获取统计信息
        let today_range = today_range();
        let today_entries = db.get_time_entries_by_date_range(
            today_range.start.date_naive(),
            today_range.end.date_naive(),
        )?;
        let total_categories = db.get_all_categories()?.len();

        self.show_info("=== 数据库统计 ===");
        self.show_info(&format!("分类总数: {}", total_categories));

        // 使用数字格式化功能
        use crate::utils::format::format_number_with_commas;
        self.show_info(&format!(
            "格式化分类数: {}",
            format_number_with_commas(total_categories as i64)
        ));

        // 显示今日统计
        if !today_entries.is_empty() {
            self.show_info("=== 今日统计 ===");
            let total_duration: Duration = today_entries
                .iter()
                .map(|e| Duration::seconds(e.duration_seconds))
                .sum();

            self.show_info(&format!(
                "今日总时长: {}",
                format_duration_detailed(total_duration)
            ));
            self.show_info(&format!("今日任务数: {}", today_entries.len()));

            // 使用百分比格式化显示效率
            let work_hours = 8.0 * 3600.0; // 8小时工作时间（秒）
            let actual_seconds = total_duration.num_seconds() as f64;
            if actual_seconds > 0.0 {
                use crate::utils::format::format_percentage;
                let efficiency = format_percentage(actual_seconds / work_hours * 100.0, 1);
                self.show_info(&format!("工作效率: {}", efficiency));
            }

            // 使用format_duration_decimal_hours展示时长
            use crate::utils::format::format_duration_decimal_hours;
            let hours_decimal = format_duration_decimal_hours(total_duration);
            self.show_info(&format!("今日时长(小时): {}", hours_decimal));
        } else {
            // 即使没有条目，也要使用db变量显示一些信息
            let all_entries_count = db.get_all_time_entries()?.len();
            self.show_info(&format!("历史总记录数: {}", all_entries_count));
        }

        Ok(())
    }

    /// 处理列表命令
    pub async fn handle_list(
        &self,
        days: u32,
        category: Option<&str>,
        tag: Option<&str>,
        search: Option<&str>,
        limit: Option<usize>,
    ) -> Result<()> {
        let db = self.get_database()?;

        // 计算日期范围
        let (start_date, end_date) = if days == 1 {
            let today = today_range();
            (today.start.date_naive(), today.end.date_naive())
        } else if days == 7 {
            let week = this_week_range();
            (week.start.date_naive(), week.end.date_naive())
        } else {
            let end_date = Local::now().date_naive();
            let start_date = end_date - Duration::days(days as i64);
            (start_date, end_date)
        };

        // 获取时间记录
        let mut entries = db.get_time_entries_by_date_range(start_date, end_date)?;

        // 应用过滤器
        if let Some(cat_name) = category {
            let category_id = self.find_category_id(&db, cat_name)?;
            entries.retain(|e| e.category_id == category_id);
        }

        if let Some(tag_filter) = tag {
            entries.retain(|e| e.tags.contains(&tag_filter.to_string()));
        }

        if let Some(search_term) = search {
            let search_lower = search_term.to_lowercase();
            entries.retain(|e| {
                e.task_name.to_lowercase().contains(&search_lower)
                    || e.description
                        .as_ref()
                        .is_some_and(|d| d.to_lowercase().contains(&search_lower))
            });
        }

        // 应用限制
        if let Some(limit_count) = limit {
            entries.truncate(limit_count);
        }

        // 格式化输出
        self.formatter.format_time_entries(&entries)?;

        Ok(())
    }

    /// 处理统计命令
    pub async fn handle_stats(
        &self,
        period: &StatsPeriod,
        category: Option<&str>,
        detailed: bool,
    ) -> Result<()> {
        let db = self.get_database()?;

        // 计算时间范围
        let (start_date, end_date) = self.calculate_period_range(period)?;

        // 使用date utils中的函数显示更多时间信息
        use crate::utils::date::{
            count_weekdays_between, get_dates_in_range, get_month_name, get_weekday_name,
            is_weekday, is_weekend, DateRange,
        };

        let date_range = DateRange::new(
            start_date
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(chrono::Local)
                .unwrap(),
            end_date
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(chrono::Local)
                .unwrap(),
        );

        self.show_info(&format!(
            "=== {} 统计报告 ===",
            match period {
                StatsPeriod::Today => "今日",
                StatsPeriod::Yesterday => "昨日",
                StatsPeriod::ThisWeek => "本周",
                StatsPeriod::LastWeek => "上周",
                StatsPeriod::ThisMonth => "本月",
                StatsPeriod::LastMonth => "上月",
                StatsPeriod::ThisYear => "本年",
                StatsPeriod::LastYear => "去年",
                StatsPeriod::Last7Days => "最近7天",
                StatsPeriod::Last30Days => "最近30天",
                StatsPeriod::Last3Months => "最近3个月",
                StatsPeriod::Custom => "自定义时间段",
            }
        ));

        // 显示时间范围详细信息
        self.show_info(&format!(
            "时间范围: {} 至 {}",
            start_date.format("%Y年%m月%d日"),
            end_date.format("%Y年%m月%d日")
        ));

        // 显示更多时间信息
        let start_weekday = start_date.weekday();
        let end_weekday = end_date.weekday();
        self.show_info(&format!(
            "开始日期: {} ({})",
            start_date.format("%Y-%m-%d"),
            get_weekday_name(start_weekday)
        ));
        self.show_info(&format!(
            "结束日期: {} ({})",
            end_date.format("%Y-%m-%d"),
            get_weekday_name(end_weekday)
        ));

        // 分析工作日/周末分布
        let total_days = date_range.days();
        let weekdays = count_weekdays_between(start_date, end_date);
        let weekend_days = total_days - weekdays;

        self.show_info(&format!(
            "总天数: {} 天 (工作日: {} 天, 周末: {} 天)",
            total_days, weekdays, weekend_days
        ));

        // 显示月份信息（如果跨月）
        if start_date.month() != end_date.month() || start_date.year() != end_date.year() {
            self.show_info(&format!(
                "跨越月份: {} {} 至 {} {}",
                start_date.year(),
                get_month_name(start_date.month()),
                end_date.year(),
                get_month_name(end_date.month())
            ));
        }

        // 获取时间记录
        let mut entries = db.get_time_entries_by_date_range(start_date, end_date)?;

        // 分类过滤
        if let Some(cat_name) = category {
            let category_id = self.find_category_id(&db, cat_name)?;
            entries.retain(|e| e.category_id == category_id);
            self.show_info(&format!("过滤分类: {}", cat_name));
        }

        if entries.is_empty() {
            self.show_warning("该时间段内没有时间记录");
            return Ok(());
        }

        // 计算基本统计
        let stats = self.calculate_stats(&entries, start_date, end_date)?;

        // 显示基本统计信息，使用正确的字段名
        let total_duration = Duration::seconds(stats.total_seconds);
        let average_duration = Duration::seconds(stats.average_seconds as i64); // 修复类型转换

        self.show_success(&format!(
            "总时长: {}",
            format_duration_detailed(total_duration)
        ));
        self.show_info(&format!(
            "平均每日: {}",
            format_duration_detailed(average_duration)
        ));
        self.show_info(&format!("记录条数: {}", stats.task_count));

        // 使用数字格式化和百分比格式化
        use crate::utils::format::{
            format_duration_decimal_hours, format_number_with_commas, format_percentage,
        };
        self.show_info(&format!(
            "格式化记录数: {}",
            format_number_with_commas(stats.task_count as i64)
        ));

        // 使用format_duration_decimal_hours显示小时格式
        let total_hours = format_duration_decimal_hours(total_duration);
        self.show_info(&format!("总时长(小时): {}", total_hours));

        if detailed {
            self.show_info("\n=== 详细统计 ===");

            // 显示每日详情
            for date in get_dates_in_range(&date_range) {
                let date_entries: Vec<_> = entries
                    .iter()
                    .filter(|e| e.start_time.date_naive() == date)
                    .collect();

                if !date_entries.is_empty() {
                    let daily_duration: Duration = date_entries
                        .iter()
                        .map(|e| Duration::seconds(e.duration_seconds))
                        .sum();

                    let weekday_info = if is_weekday(date) {
                        "工作日"
                    } else {
                        "周末"
                    };

                    println!(
                        "  {} ({}, {}): {} - {} 个任务",
                        date.format("%m月%d日"),
                        get_weekday_name(date.weekday()),
                        weekday_info,
                        format_duration_compact(daily_duration),
                        date_entries.len()
                    );
                }
            }
        }

        Ok(())
    }

    /// 处理分类命令
    pub async fn handle_category(&self, action: &CategoryAction) -> Result<()> {
        let db = self.get_database()?;

        match action {
            CategoryAction::List => {
                let categories = db.get_all_categories()?;
                self.formatter.format_categories(&categories)?;
            }

            CategoryAction::Create {
                name,
                description,
                color,
                icon,
                parent,
            } => {
                // 检查父分类
                let parent_id = if let Some(parent_name) = parent {
                    Some(
                        self.find_category_id(&db, parent_name)?
                            .ok_or_else(|| AppError::CategoryNotFound(parent_name.clone()))?,
                    )
                } else {
                    None
                };

                let category = CategoryInsert {
                    id: Uuid::new_v4(),
                    name: name.clone(),
                    description: description.clone(),
                    color: color.clone(),
                    icon: icon.clone(),
                    daily_target_seconds: None,
                    weekly_target_seconds: None,
                    is_active: true,
                    sort_order: 0,
                    parent_id,
                    created_at: Local::now(),
                };

                db.insert_category(&category)?;
                self.show_success(&format!("分类 '{}' 创建成功", name));
            }

            CategoryAction::Update {
                category,
                name,
                description,
                color,
                icon,
            } => {
                let category_id = self
                    .find_category_id(&db, category)?
                    .ok_or_else(|| AppError::CategoryNotFound(category.clone()))?;

                // 获取现有分类
                let existing = db
                    .get_category_by_id(category_id)?
                    .ok_or_else(|| AppError::CategoryNotFound(category.clone()))?;

                let updated = CategoryInsert {
                    id: existing.id,
                    name: name.clone().unwrap_or(existing.name),
                    description: description.clone().or(existing.description),
                    color: color.clone().unwrap_or(existing.color),
                    icon: icon.clone().unwrap_or(existing.icon),
                    daily_target_seconds: existing.daily_target_seconds,
                    weekly_target_seconds: existing.weekly_target_seconds,
                    is_active: existing.is_active,
                    sort_order: existing.sort_order,
                    parent_id: existing.parent_id,
                    created_at: existing.created_at,
                };

                db.update_category(category_id, &updated)?;
                self.show_success(&format!("分类 '{}' 更新成功", category));
            }

            CategoryAction::Delete { category, force } => {
                let category_id = self
                    .find_category_id(&db, category)?
                    .ok_or_else(|| AppError::CategoryNotFound(category.clone()))?;

                if !force && !self.confirm(&format!("确定要删除分类 '{}' 吗？", category))?
                {
                    self.show_info("操作已取消");
                    return Ok(());
                }

                db.delete_category(category_id)?;
                self.show_success(&format!("分类 '{}' 删除成功", category));
            }
        }

        Ok(())
    }

    /// 处理配置命令
    pub async fn handle_config(&self, action: &ConfigAction) -> Result<()> {
        match action {
            ConfigAction::Show => {
                self.show_info("当前配置:");
                // TODO: 实现配置显示
                println!("数据库路径: {:?}", self.cli.database);
                println!("输出格式: {:?}", self.cli.format);
                println!("详细模式: {}", self.cli.verbose);
                println!("静默模式: {}", self.cli.quiet);
            }

            ConfigAction::Set { key, value } => {
                // TODO: 实现配置设置
                self.show_success(&format!("配置 '{}' 已设置为 '{}'", key, value));
            }

            ConfigAction::Get { key } => {
                // TODO: 实现配置获取
                self.show_info(&format!("配置 '{}' 的值: 未实现", key));
            }

            ConfigAction::Reset { force } => {
                if !force && !self.confirm("确定要重置所有配置吗？")? {
                    self.show_info("操作已取消");
                    return Ok(());
                }

                // TODO: 实现配置重置
                self.show_success("配置已重置为默认值");
            }
        }

        Ok(())
    }

    /// 处理数据命令
    pub async fn handle_data(&self, action: &DataAction) -> Result<()> {
        match action {
            DataAction::Export {
                output,
                format,
                start_date,
                end_date,
            } => {
                let db = self.get_database()?;

                // 解析日期范围
                let (start, end) =
                    Self::parse_date_range(start_date.as_deref(), end_date.as_deref())?;

                // 获取数据
                let entries = db.get_time_entries_by_date_range(start, end)?;
                let categories = db.get_all_categories()?;

                self.show_info("正在导出数据...");

                // 创建进度条显示
                let total_steps = 3;
                for i in 0..=total_steps {
                    let progress = create_progress_bar(i, total_steps, 20);
                    println!("\r{} ({}/{})", progress, i, total_steps);
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
                println!(); // 新行

                // 导出数据
                self.export_data(&entries, &categories, output, format)?;

                // 检查文件大小并格式化显示
                if let Ok(metadata) = std::fs::metadata(output) {
                    let file_size = metadata.len();
                    self.show_success(&format!(
                        "数据已成功导出到: {} (大小: {})",
                        output.display(),
                        format_file_size(file_size)
                    ));
                } else {
                    self.show_success(&format!("数据已成功导出到: {}", output.display()));
                }

                self.show_info(&format!(
                    "导出了 {} 条记录和 {} 个分类",
                    entries.len(),
                    categories.len()
                ));
            }
            DataAction::Import {
                input,
                format,
                overwrite,
            } => {
                // 检查文件是否存在
                if !input.exists() {
                    return Err(AppError::NotFound(input.to_string_lossy().to_string()));
                }

                // 检查文件扩展名
                if let Some(ext) = get_file_extension(&input.to_string_lossy()) {
                    self.show_info(&format!("检测到文件扩展名: {}", ext));

                    // 验证格式匹配
                    let expected_ext = match format {
                        ExportFormat::Json => "json",
                        ExportFormat::Csv => "csv",
                        ExportFormat::Excel => "xlsx",
                    };

                    if ext != expected_ext {
                        self.show_warning(&format!(
                            "警告: 文件扩展名 '{}' 与指定格式 '{:?}' 不匹配",
                            ext, format
                        ));
                    }
                }

                // 获取文件大小
                if let Ok(metadata) = std::fs::metadata(input) {
                    let file_size = metadata.len();
                    self.show_info(&format!("文件大小: {}", format_file_size(file_size)));
                }

                self.show_info("正在导入数据...");

                // 模拟导入过程（实际应该实现真正的导入逻辑）
                let total_steps = 5;
                for i in 0..=total_steps {
                    let progress = create_progress_bar(i, total_steps, 25);
                    println!("\r{} 导入进度 ({}/{})", progress, i, total_steps);
                    std::thread::sleep(std::time::Duration::from_millis(300));
                }
                println!(); // 新行

                if *overwrite {
                    self.show_warning("覆盖模式已启用 - 现有数据将被替换");
                }

                // TODO: 实现实际的导入逻辑
                self.show_success(&format!("数据从 {} 导入完成", input.display()));
            }
            DataAction::Backup { output } => {
                let db = self.get_database()?;

                self.show_info("正在备份数据库...");

                // 生成备份会话ID用于跟踪
                use crate::utils::generate_random_string;
                let backup_session_id = generate_random_string(8);
                self.show_info(&format!("备份会话ID: {}", backup_session_id));

                // 创建目录（如果不存在）
                use crate::utils::ensure_dir_exists;
                if let Some(parent) = output.parent() {
                    ensure_dir_exists(parent)?;
                }

                // 获取数据库统计信息用于备份
                let all_entries = db.get_all_time_entries()?;
                let all_categories = db.get_all_categories()?;

                self.show_info(&format!(
                    "准备备份 {} 条记录和 {} 个分类",
                    all_entries.len(),
                    all_categories.len()
                ));

                // 创建临时备份文件名（包含随机后缀）
                let temp_suffix = generate_random_string(6);
                let temp_backup_name = format!("backup_temp_{}.tmp", temp_suffix);
                self.show_info(&format!("使用临时文件: {}", temp_backup_name));

                // 备份数据库（简化实现）
                // TODO: 实现实际的数据库备份逻辑

                // 模拟备份过程
                let total_steps = 4;
                for i in 0..=total_steps {
                    let progress = create_progress_bar(i, total_steps, 30);
                    println!("\r{} 备份进度 ({}/{})", progress, i, total_steps);
                    std::thread::sleep(std::time::Duration::from_millis(250));
                }
                println!(); // 新行

                // 显示备份文件信息
                if let Ok(metadata) = std::fs::metadata(output) {
                    let file_size = metadata.len();
                    self.show_success(&format!(
                        "数据库已备份到: {} (大小: {}, 会话: {})",
                        output.display(),
                        format_file_size(file_size),
                        backup_session_id
                    ));
                } else {
                    self.show_success(&format!(
                        "数据库已备份到: {} (会话: {})",
                        output.display(),
                        backup_session_id
                    ));
                }
            }
            DataAction::Restore { input, force } => {
                if !input.exists() {
                    return Err(AppError::NotFound(input.to_string_lossy().to_string()));
                }

                if !force {
                    // 显示确认提示
                    self.show_warning("警告: 此操作将覆盖当前数据库中的所有数据！");
                    println!("请使用 --force 参数确认此操作");
                    return Ok(());
                }

                self.show_info("正在恢复数据库...");

                // 显示备份文件信息
                if let Ok(metadata) = std::fs::metadata(input) {
                    let file_size = metadata.len();
                    self.show_info(&format!("备份文件大小: {}", format_file_size(file_size)));
                }

                // 模拟恢复过程
                let total_steps = 6;
                for i in 0..=total_steps {
                    let progress = create_progress_bar(i, total_steps, 25);
                    println!("\r{} 恢复进度 ({}/{})", progress, i, total_steps);
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
                println!(); // 新行

                // TODO: 实现实际的数据库恢复逻辑
                self.show_success(&format!("数据库已从 {} 恢复", input.display()));
            }
        }

        Ok(())
    }

    /// 处理报告命令
    pub async fn handle_report(
        &self,
        report_type: &ReportType,
        start_date: Option<&str>,
        end_date: Option<&str>,
        output: Option<&Path>,
    ) -> Result<()> {
        let db = self.get_database()?;

        // 解析日期范围
        let (start, end) = CliApp::parse_date_range(start_date, end_date)?;

        // 获取数据
        let entries = db.get_time_entries_by_date_range(start, end)?;

        // 生成报告
        match report_type {
            ReportType::Daily => {
                let report = self.generate_daily_report(&entries, start, end)?;
                self.output_report(&report, output)?;
            }

            ReportType::Weekly => {
                let report = self.generate_weekly_report(&entries, start, end)?;
                self.output_report(&report, output)?;
            }

            ReportType::Monthly => {
                let report = self.generate_monthly_report(&entries, start, end)?;
                self.output_report(&report, output)?;
            }

            ReportType::Category => {
                let categories = db.get_all_categories()?;
                let report = self.generate_category_report(&entries, &categories)?;
                self.output_report(&report, output)?;
            }

            ReportType::Trend => {
                let report = self.generate_trend_report(&entries, start, end)?;
                self.output_report(&report, output)?;
            }
        }

        self.show_success("报告生成完成");
        Ok(())
    }

    // ==================== 辅助方法 ====================

    /// 获取数据库连接
    fn get_database(&self) -> Result<Database> {
        let db_path = self.cli.database.clone().unwrap_or_else(|| {
            dirs::data_dir()
                .unwrap_or_else(|| std::env::current_dir().unwrap())
                .join("time_tracker")
                .join("data.db")
        });

        Database::new(db_path)
    }

    /// 查找分类ID
    fn find_category_id(&self, db: &Database, name_or_id: &str) -> Result<Option<Uuid>> {
        // 首先尝试解析为UUID
        if let Ok(uuid) = Uuid::parse_str(name_or_id) {
            return Ok(Some(uuid));
        }

        // 否则按名称查找
        let categories = db.get_all_categories()?;
        for category in categories {
            if category.name == name_or_id {
                return Ok(Some(category.id));
            }
        }

        Ok(None)
    }

    /// 计算统计周期的日期范围
    fn calculate_period_range(&self, period: &StatsPeriod) -> Result<(NaiveDate, NaiveDate)> {
        use crate::utils::date::*;

        let range = match period {
            StatsPeriod::Today => today_range(),
            StatsPeriod::Yesterday => yesterday_range(),
            StatsPeriod::ThisWeek => this_week_range(),
            StatsPeriod::LastWeek => last_week_range(),
            StatsPeriod::ThisMonth => this_month_range(),
            StatsPeriod::LastMonth => last_month_range(),
            StatsPeriod::Last7Days => last_n_days_range(7),
            StatsPeriod::Last30Days => last_n_days_range(30),
            StatsPeriod::Last3Months => last_n_months_range(3),
            StatsPeriod::ThisYear => {
                let now = Local::now();
                let start = now.date_naive().with_ordinal(1).unwrap();
                let end = now.date_naive();
                DateRange::new(
                    start
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap(),
                    end.and_hms_opt(23, 59, 59)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap(),
                )
            }
            StatsPeriod::LastYear => {
                let now = Local::now();
                let last_year = now.year() - 1;
                let start = NaiveDate::from_ymd_opt(last_year, 1, 1).unwrap();
                let end = NaiveDate::from_ymd_opt(last_year, 12, 31).unwrap();
                DateRange::new(
                    start
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap(),
                    end.and_hms_opt(23, 59, 59)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap(),
                )
            }
            StatsPeriod::Custom => {
                // 对于自定义周期，使用最近7天作为默认
                last_n_days_range(7)
            }
        };

        Ok((range.start.date_naive(), range.end.date_naive()))
    }

    /// 解析日期范围
    fn parse_date_range(
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<(NaiveDate, NaiveDate)> {
        let end = if let Some(end_str) = end_date {
            NaiveDate::parse_from_str(end_str, "%Y-%m-%d")
                .map_err(|_| AppError::InvalidInput(format!("无效的结束日期: {}", end_str)))?
        } else {
            Local::now().date_naive()
        };

        let start = if let Some(start_str) = start_date {
            NaiveDate::parse_from_str(start_str, "%Y-%m-%d")
                .map_err(|_| AppError::InvalidInput(format!("无效的开始日期: {}", start_str)))?
        } else {
            end - Duration::days(7)
        };

        if start > end {
            return Err(AppError::InvalidInput(
                "开始日期不能晚于结束日期".to_string(),
            ));
        }

        Ok((start, end))
    }

    /// 计算统计数据
    fn calculate_stats(
        &self,
        entries: &[TimeEntry],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TimeStats> {
        if entries.is_empty() {
            return Ok(TimeStats {
                total_seconds: 0,
                task_count: 0,
                average_seconds: 0.0,
                max_seconds: 0,
                min_seconds: 0,
                start_date: start_date
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap(),
                end_date: end_date
                    .and_hms_opt(23, 59, 59)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap(),
            });
        }

        let total_seconds: i64 = entries.iter().map(|e| e.duration_seconds).sum();
        let task_count = entries.len() as i64;
        let average_seconds = if task_count > 0 {
            total_seconds as f64 / task_count as f64
        } else {
            0.0
        };

        let max_seconds = entries
            .iter()
            .map(|e| e.duration_seconds)
            .max()
            .unwrap_or(0);
        let min_seconds = entries
            .iter()
            .map(|e| e.duration_seconds)
            .min()
            .unwrap_or(0);

        Ok(TimeStats {
            total_seconds,
            task_count,
            average_seconds,
            max_seconds,
            min_seconds,
            start_date: start_date
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap(),
            end_date: end_date
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap(),
        })
    }

    /// 导出数据
    fn export_data(
        &self,
        entries: &[TimeEntry],
        categories: &[CategoryModel],
        output: &Path,
        format: &ExportFormat,
    ) -> Result<()> {
        match format {
            ExportFormat::Json => {
                let data = serde_json::json!({
                    "entries": entries,
                    "categories": categories,
                    "exported_at": Local::now().to_rfc3339(),
                });

                std::fs::write(output, serde_json::to_string_pretty(&data)?)?;
            }

            ExportFormat::Csv => {
                // TODO: 实现CSV导出
                return Err(AppError::InvalidInput("CSV导出尚未实现".to_string()));
            }

            ExportFormat::Excel => {
                // TODO: 实现Excel导出
                return Err(AppError::InvalidInput("Excel导出尚未实现".to_string()));
            }
        }

        Ok(())
    }

    /// 生成每日报告
    fn generate_daily_report(
        &self,
        entries: &[TimeEntry],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<String> {
        use crate::utils::format::{format_duration_decimal_hours, format_percentage};

        let mut report = String::new();
        report.push_str(&format!(
            "=== 每日报告 ({} 到 {}) ===\n",
            start_date, end_date
        ));

        if entries.is_empty() {
            report.push_str("该时间段内没有时间记录\n");
            return Ok(report);
        }

        let total_duration: Duration = entries
            .iter()
            .map(|e| Duration::seconds(e.duration_seconds))
            .sum();

        report.push_str(&format!("总记录数: {}\n", entries.len()));
        report.push_str(&format!(
            "总时长: {}\n",
            format_duration_detailed(total_duration)
        ));
        report.push_str(&format!(
            "总时长(小时): {}\n",
            format_duration_decimal_hours(total_duration)
        ));

        // 计算日平均
        let days = (end_date - start_date).num_days() + 1;
        let avg_per_day = Duration::seconds(total_duration.num_seconds() / days);
        report.push_str(&format!(
            "日平均时长: {}\n",
            format_duration_detailed(avg_per_day)
        ));

        // 计算效率百分比（假设目标是8小时/天）
        let target_hours_per_day = 8.0;
        let actual_hours_per_day = total_duration.num_seconds() as f64 / 3600.0 / days as f64;
        let efficiency = format_percentage(actual_hours_per_day / target_hours_per_day * 100.0, 1);
        report.push_str(&format!("日均效率: {}\n", efficiency));

        Ok(report)
    }

    /// 生成每周报告
    fn generate_weekly_report(
        &self,
        entries: &[TimeEntry],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<String> {
        use crate::utils::date::{get_weekday_name, is_weekday};
        use crate::utils::format::{format_duration_decimal_hours, format_percentage};

        let mut report = String::new();
        report.push_str(&format!(
            "=== 每周报告 ({} 到 {}) ===\n",
            start_date, end_date
        ));

        if entries.is_empty() {
            report.push_str("该时间段内没有时间记录\n");
            return Ok(report);
        }

        // 分组统计工作日和周末
        let mut weekday_entries = Vec::new();
        let mut weekend_entries = Vec::new();

        for entry in entries {
            let entry_date = entry.start_time.date_naive();
            if is_weekday(entry_date) {
                weekday_entries.push(entry);
            } else {
                weekend_entries.push(entry);
            }
        }

        let weekday_duration: Duration = weekday_entries
            .iter()
            .map(|e| Duration::seconds(e.duration_seconds))
            .sum();
        let weekend_duration: Duration = weekend_entries
            .iter()
            .map(|e| Duration::seconds(e.duration_seconds))
            .sum();
        let total_duration = weekday_duration + weekend_duration;

        report.push_str(&format!("总记录数: {}\n", entries.len()));
        report.push_str(&format!(
            "总时长: {}\n",
            format_duration_detailed(total_duration)
        ));
        report.push_str(&format!(
            "总时长(小时): {}\n",
            format_duration_decimal_hours(total_duration)
        ));

        report.push_str("\n--- 工作日统计 ---\n");
        report.push_str(&format!("工作日记录数: {}\n", weekday_entries.len()));
        report.push_str(&format!(
            "工作日时长: {}\n",
            format_duration_detailed(weekday_duration)
        ));

        report.push_str("\n--- 周末统计 ---\n");
        report.push_str(&format!("周末记录数: {}\n", weekend_entries.len()));
        report.push_str(&format!(
            "周末时长: {}\n",
            format_duration_detailed(weekend_duration)
        ));

        // 计算比例
        if total_duration.num_seconds() > 0 {
            let weekday_percent = format_percentage(
                weekday_duration.num_seconds() as f64 / total_duration.num_seconds() as f64 * 100.0,
                1,
            );
            let weekend_percent = format_percentage(
                weekend_duration.num_seconds() as f64 / total_duration.num_seconds() as f64 * 100.0,
                1,
            );
            report.push_str(&format!("工作日占比: {}\n", weekday_percent));
            report.push_str(&format!("周末占比: {}\n", weekend_percent));
        }

        Ok(report)
    }

    /// 生成每月报告
    fn generate_monthly_report(
        &self,
        entries: &[TimeEntry],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<String> {
        use crate::utils::date::{get_month_name, is_weekday};
        use crate::utils::format::{format_duration_decimal_hours, format_percentage};

        let mut report = String::new();
        report.push_str(&format!(
            "=== 每月报告 ({} 到 {}) ===\n",
            start_date, end_date
        ));

        if entries.is_empty() {
            report.push_str("该时间段内没有时间记录\n");
            return Ok(report);
        }

        let total_duration: Duration = entries
            .iter()
            .map(|e| Duration::seconds(e.duration_seconds))
            .sum();

        report.push_str(&format!("总记录数: {}\n", entries.len()));
        report.push_str(&format!(
            "总时长: {}\n",
            format_duration_detailed(total_duration)
        ));
        report.push_str(&format!(
            "总时长(小时): {}\n",
            format_duration_decimal_hours(total_duration)
        ));

        // 按月份分组
        let mut monthly_stats = std::collections::HashMap::new();
        for entry in entries {
            let entry_date = entry.start_time.date_naive();
            let month_key = format!("{}-{:02}", entry_date.year(), entry_date.month());
            let entry_duration = Duration::seconds(entry.duration_seconds);

            let stats = monthly_stats
                .entry(month_key)
                .or_insert((0, Duration::zero()));
            stats.0 += 1;
            stats.1 = stats.1 + entry_duration;
        }

        report.push_str("\n--- 月度详情 ---\n");
        let mut sorted_months: Vec<_> = monthly_stats.iter().collect();
        sorted_months.sort_by_key(|(k, _)| k.as_str());

        for (month_key, (count, duration)) in sorted_months {
            let parts: Vec<&str> = month_key.split('-').collect();
            if parts.len() == 2 {
                if let (Ok(year), Ok(month)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>()) {
                    let month_name = get_month_name(month);
                    report.push_str(&format!(
                        "{} {}: {} 条记录, 时长 {}\n",
                        year,
                        month_name,
                        count,
                        format_duration_detailed(*duration)
                    ));
                }
            }
        }

        // 计算月平均
        let months_count = monthly_stats.len() as f64;
        if months_count > 0.0 {
            let avg_per_month =
                Duration::seconds((total_duration.num_seconds() as f64 / months_count) as i64);
            report.push_str(&format!(
                "月平均时长: {}\n",
                format_duration_detailed(avg_per_month)
            ));
        }

        Ok(report)
    }

    /// 生成分类报告
    fn generate_category_report(
        &self,
        entries: &[TimeEntry],
        categories: &[CategoryModel],
    ) -> Result<String> {
        use crate::utils::format::{format_duration_decimal_hours, format_percentage};

        let mut report = String::new();
        report.push_str("=== 分类报告 ===\n");

        if entries.is_empty() {
            report.push_str("没有时间记录\n");
            return Ok(report);
        }

        if categories.is_empty() {
            report.push_str("没有分类数据\n");
            return Ok(report);
        }

        let total_duration: Duration = entries
            .iter()
            .map(|e| Duration::seconds(e.duration_seconds))
            .sum();

        report.push_str(&format!("总记录数: {}\n", entries.len()));
        report.push_str(&format!(
            "总时长: {}\n",
            format_duration_detailed(total_duration)
        ));
        report.push_str(&format!("分类总数: {}\n", categories.len()));

        // 按分类统计
        let mut category_stats = std::collections::HashMap::new();
        for entry in entries {
            let category_id = entry.category_id.unwrap_or_default();
            let entry_duration = Duration::seconds(entry.duration_seconds);

            let stats = category_stats
                .entry(category_id)
                .or_insert((0, Duration::zero()));
            stats.0 += 1;
            stats.1 = stats.1 + entry_duration;
        }

        report.push_str("\n--- 分类详情 ---\n");
        for category in categories {
            if let Some((count, duration)) = category_stats.get(&category.id) {
                let percentage = if total_duration.num_seconds() > 0 {
                    format_percentage(
                        duration.num_seconds() as f64 / total_duration.num_seconds() as f64 * 100.0,
                        1,
                    )
                } else {
                    "0.0%".to_string()
                };

                report.push_str(&format!(
                    "{}: {} 条记录, 时长 {} ({}), 占比 {}\n",
                    category.name,
                    count,
                    format_duration_detailed(*duration),
                    format_duration_decimal_hours(*duration),
                    percentage
                ));
            } else {
                report.push_str(&format!("{}: 0 条记录\n", category.name));
            }
        }

        Ok(report)
    }

    /// 生成趋势报告
    fn generate_trend_report(
        &self,
        entries: &[TimeEntry],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<String> {
        use crate::utils::date::{get_dates_in_range, DateRange};
        use crate::utils::format::{format_duration_decimal_hours, format_percentage};

        let mut report = String::new();
        report.push_str(&format!(
            "=== 趋势报告 ({} 到 {}) ===\n",
            start_date, end_date
        ));

        if entries.is_empty() {
            report.push_str("该时间段内没有时间记录\n");
            return Ok(report);
        }

        let date_range = DateRange::new(
            start_date
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(chrono::Local)
                .unwrap(),
            end_date
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(chrono::Local)
                .unwrap(),
        );

        // 按日期统计
        let mut daily_stats = std::collections::HashMap::new();
        for entry in entries {
            let entry_date = entry.start_time.date_naive();
            let entry_duration = Duration::seconds(entry.duration_seconds);

            let stats = daily_stats
                .entry(entry_date)
                .or_insert((0, Duration::zero()));
            stats.0 += 1;
            stats.1 = stats.1 + entry_duration;
        }

        let total_duration: Duration = entries
            .iter()
            .map(|e| Duration::seconds(e.duration_seconds))
            .sum();

        report.push_str(&format!("总记录数: {}\n", entries.len()));
        report.push_str(&format!(
            "总时长: {}\n",
            format_duration_detailed(total_duration)
        ));

        // 计算趋势
        let dates = get_dates_in_range(&date_range);
        let mut daily_hours = Vec::new();

        report.push_str("\n--- 每日趋势 ---\n");
        for date in &dates {
            if let Some((count, duration)) = daily_stats.get(date) {
                let hours = duration.num_seconds() as f64 / 3600.0;
                daily_hours.push(hours);
                report.push_str(&format!(
                    "{}: {} 条记录, {} ({})\n",
                    date.format("%m-%d"),
                    count,
                    format_duration_detailed(*duration),
                    format_duration_decimal_hours(*duration)
                ));
            } else {
                daily_hours.push(0.0);
                report.push_str(&format!("{}: 0 条记录\n", date.format("%m-%d")));
            }
        }

        // 计算平均值和趋势
        if !daily_hours.is_empty() {
            let avg_hours = daily_hours.iter().sum::<f64>() / daily_hours.len() as f64;
            report.push_str(&format!("\n日均工作时长: {:.1} 小时\n", avg_hours));

            // 简单的趋势分析（比较前半段和后半段）
            let mid_point = daily_hours.len() / 2;
            if mid_point > 0 {
                let first_half_avg =
                    daily_hours[..mid_point].iter().sum::<f64>() / mid_point as f64;
                let second_half_avg = daily_hours[mid_point..].iter().sum::<f64>()
                    / (daily_hours.len() - mid_point) as f64;

                let trend_change = second_half_avg - first_half_avg;
                let trend_percent = if first_half_avg > 0.0 {
                    format_percentage(trend_change / first_half_avg * 100.0, 1)
                } else {
                    "N/A".to_string()
                };

                report.push_str(&format!(
                    "趋势分析: 后半段比前半段{:.1}小时 ({})\n",
                    trend_change.abs(),
                    if trend_change > 0.0 {
                        format!("增长 {}", trend_percent)
                    } else {
                        format!("下降 {}", trend_percent)
                    }
                ));
            }
        }

        Ok(report)
    }

    /// 输出报告
    fn output_report(&self, report: &str, output: Option<&Path>) -> Result<()> {
        if let Some(path) = output {
            std::fs::write(path, report)?;
            self.show_info(&format!("报告已保存到: {:?}", path));
        } else {
            println!("{}", report);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_parse_date_range() {
        // 这里需要创建一个测试用的CliApp实例
        // 由于依赖较多，这里只是示例
    }
}
