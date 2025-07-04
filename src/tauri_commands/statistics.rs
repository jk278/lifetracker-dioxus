//! # 统计分析命令模块
//!
//! 负责处理各种统计数据的生成和查询

use super::*;

// ========== 统计分析命令 ==========

/// 获取统计数据
#[tauri::command]
pub async fn get_statistics(
    state: State<'_, AppState>,
    period: Option<String>, // "today", "week", "month", "all"
) -> Result<StatisticsDto, String> {
    // TODO: 实现实际的统计数据查询逻辑
    let stats = StatisticsDto {
        today: DayStatDto {
            date: Local::now().format("%Y-%m-%d").to_string(),
            total_seconds: 7200, // 2小时
            task_count: 3,
            active_categories: 2,
            most_productive_hour: Some(14), // 下午2点
        },
        this_week: PeriodStatDto {
            total_seconds: 36000, // 10小时
            task_count: 15,
            active_days: 5,
            average_daily_seconds: 7200,
        },
        this_month: PeriodStatDto {
            total_seconds: 144000, // 40小时
            task_count: 60,
            active_days: 20,
            average_daily_seconds: 7200,
        },
        all_time: PeriodStatDto {
            total_seconds: 720000, // 200小时
            task_count: 300,
            active_days: 100,
            average_daily_seconds: 7200,
        },
        category_stats: vec![
            CategoryStatDto {
                category_id: "cat1".to_string(),
                category_name: "工作".to_string(),
                total_seconds: 18000,
                task_count: 10,
                percentage: 60.0,
            },
            CategoryStatDto {
                category_id: "cat2".to_string(),
                category_name: "学习".to_string(),
                total_seconds: 12000,
                task_count: 8,
                percentage: 40.0,
            },
        ],
        daily_trend: vec![], // TODO: 添加7天的趋势数据
    };

    Ok(stats)
}

/// 获取财务统计
#[tauri::command]
pub async fn get_financial_stats(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<FinancialStatsDto, String> {
    log::debug!(
        "[CMD] get_financial_stats: Attempting for {} to {}",
        start_date,
        end_date
    );
    let storage = &state.storage;

    let start_date_naive = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|_| "无效的开始日期格式")?;
    let end_date_naive = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|_| "无效的结束日期格式")?;

    log::info!("[CMD] get_financial_stats: Fetching stats from database");
    let stats_from_db = storage
        .get_database()
        .get_transaction_statistics(start_date_naive, end_date_naive)
        .map_err(|e| e.to_string())?;
    log::debug!("[CMD] get_financial_stats: Stats fetched, mapping DTO.");

    let stats_dto = FinancialStatsDto {
        total_income: stats_from_db.total_income,
        total_expense: stats_from_db.total_expense,
        net_income: stats_from_db.net_income,
        account_balance: stats_from_db.account_balance,
        transaction_count: stats_from_db.transaction_count,
        period_start: stats_from_db.period_start.format("%Y-%m-%d").to_string(),
        period_end: stats_from_db.period_end.format("%Y-%m-%d").to_string(),
        currency: stats_from_db.currency,
    };

    log::debug!("财务统计获取成功");
    Ok(stats_dto)
}

/// 月度趋势数据传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyTrendDto {
    pub month: String,
    pub income: f64,
    pub expense: f64,
    pub net: f64,
}

/// 获取月度趋势数据
#[tauri::command]
pub async fn get_monthly_trend(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<MonthlyTrendDto>, String> {
    log::debug!(
        "[CMD] get_monthly_trend: Attempting for {} to {}",
        start_date,
        end_date
    );

    let storage = &state.storage;

    let start_date_naive = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|_| "无效的开始日期格式")?;
    let end_date_naive = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|_| "无效的结束日期格式")?;

    log::info!("[CMD] get_monthly_trend: Fetching transactions from database");

    // 获取指定时间范围内的所有交易
    let transactions = storage
        .get_database()
        .get_transactions_by_date_range(start_date_naive, end_date_naive)
        .map_err(|e| e.to_string())?;

    log::debug!(
        "[CMD] get_monthly_trend: Found {} transactions",
        transactions.len()
    );

    // 使用 AnalyticsManager 生成月度趋势
    let analytics = crate::core::accounting::AnalyticsManager::new();
    let monthly_trend = analytics
        .generate_monthly_trend(&transactions, start_date_naive, end_date_naive)
        .map_err(|e| e.to_string())?;

    // 转换为 DTO
    let trend_dto: Vec<MonthlyTrendDto> = monthly_trend
        .into_iter()
        .map(|trend| MonthlyTrendDto {
            month: trend.month,
            income: trend.income,
            expense: trend.expense,
            net: trend.net,
        })
        .collect();

    log::debug!(
        "[CMD] get_monthly_trend: Generated {} trend data points",
        trend_dto.len()
    );
    Ok(trend_dto)
}

/// 统一趋势数据传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendDto {
    pub label: String,
    pub income: f64,
    pub expense: f64,
    pub net: f64,
}

/// 获取收支趋势（按日/周/月）
#[tauri::command]
pub async fn get_financial_trend(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
    granularity: String, // "day" | "week" | "month"
) -> Result<Vec<TrendDto>, String> {
    let granularity = granularity.to_lowercase();
    if !["day", "week", "month"].contains(&granularity.as_str()) {
        return Err("无效的 granularity 参数，应为 day/week/month".into());
    }

    let storage = &state.storage;
    let start_date_naive = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|_| "无效的开始日期格式")?;
    let end_date_naive = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|_| "无效的结束日期格式")?;

    // 获取指定日期范围内交易
    let transactions = storage
        .get_database()
        .get_transactions_by_date_range(start_date_naive, end_date_naive)
        .map_err(|e| e.to_string())?;

    let analytics = crate::core::accounting::AnalyticsManager::new();
    let trends = match granularity.as_str() {
        "day" => analytics.generate_daily_trend(&transactions, start_date_naive, end_date_naive),
        "week" => analytics.generate_weekly_trend(&transactions, start_date_naive, end_date_naive),
        _ => analytics.generate_monthly_trend(&transactions, start_date_naive, end_date_naive),
    }
    .map_err(|e| e.to_string())?;

    // map to DTO with label
    let dtos: Vec<TrendDto> = trends
        .into_iter()
        .map(|t| TrendDto {
            label: t.month,
            income: t.income,
            expense: t.expense,
            net: t.net,
        })
        .collect();
    Ok(dtos)
}
