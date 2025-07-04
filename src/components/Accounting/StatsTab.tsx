import { invoke } from "@tauri-apps/api/core";
import type React from "react";
import { useCallback, useEffect, useState } from "react";
import type {
	FinancialStatsDto,
	TrendData,
	TrendGranularity,
} from "../../types";
import FinancialTrendChart from "./FinancialTrendChart";

interface StatsTabProps {
	financialStats: FinancialStatsDto | null;
	formatAmount: (amount: number, currency?: string) => string;
}

const StatsTab: React.FC<StatsTabProps> = ({
	financialStats,
	formatAmount,
}) => {
	// 月度趋势数据状态
	const [trendData, setTrendData] = useState<TrendData[]>([]);
	const [trendLoading, setTrendLoading] = useState(false);
	const [trendError, setTrendError] = useState<string | null>(null);

	// 图表显示控制
	const [showIncome, setShowIncome] = useState(true);
	const [showExpense, setShowExpense] = useState(true);

	// 趋势类型
	const [trendType, setTrendType] = useState<TrendGranularity>("month");

	// 获取月度趋势数据
	const fetchTrendData = useCallback(async () => {
		try {
			setTrendLoading(true);
			setTrendError(null);

			// 获取过去12个月的数据
			const endDate = new Date();
			const startDate = new Date();
			if (trendType === "day") {
				startDate.setDate(startDate.getDate() - 29); // 30 天
			} else if (trendType === "week") {
				startDate.setDate(startDate.getDate() - 7 * 11); // 12 周
			} else {
				startDate.setMonth(startDate.getMonth() - 11); // 12 个月
			}

			const trendResult = await invoke<TrendData[]>("get_financial_trend", {
				startDate: startDate.toISOString().split("T")[0],
				endDate: endDate.toISOString().split("T")[0],
				granularity: trendType,
			});

			setTrendData(trendResult);
		} catch (err) {
			console.error("获取月度趋势数据失败:", err);
			setTrendError("获取趋势数据失败");
		} finally {
			setTrendLoading(false);
		}
	}, [trendType]);

	// 组件挂载时获取趋势数据
	useEffect(() => {
		fetchTrendData();
	}, [fetchTrendData]);
	return (
		<div className="h-full overflow-y-auto mt-4 mb-0 space-y-6">
			<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
				财务统计
			</h3>

			{financialStats && (
				<div className="space-y-6">
					{/* 统计卡片 */}
					<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
						<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6">
							<h4 className="text-sm font-medium text-gray-500 dark:text-gray-400">
								总收入
							</h4>
							<p className="text-2xl font-bold text-green-600 dark:text-green-400">
								{formatAmount(financialStats.total_income)}
							</p>
						</div>
						<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6">
							<h4 className="text-sm font-medium text-gray-500 dark:text-gray-400">
								总支出
							</h4>
							<p className="text-2xl font-bold text-red-600 dark:text-red-400">
								{formatAmount(financialStats.total_expense)}
							</p>
						</div>
						<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6">
							<h4 className="text-sm font-medium text-gray-500 dark:text-gray-400">
								净收入
							</h4>
							<p className="text-2xl font-bold text-blue-600 dark:text-blue-400">
								{formatAmount(financialStats.net_income)}
							</p>
						</div>
						<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6">
							<h4 className="text-sm font-medium text-gray-500 dark:text-gray-400">
								交易笔数
							</h4>
							<p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
								{financialStats.transaction_count}
							</p>
						</div>
					</div>

					{/* 收支趋势图表 */}
					<div className="space-y-4">
						{/* 图表控制选项 */}
						<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-4">
							<div className="flex items-center justify-between">
								<h4 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
									收支趋势 (过去12个月)
								</h4>
								<div className="flex items-center space-x-4">
									<label className="flex items-center">
										<input
											type="radio"
											checked={trendType === "month"}
											onChange={() => setTrendType("month")}
											className="mr-2 rounded border-gray-300 text-green-600 focus:ring-green-500"
										/>
										<span className="text-sm text-gray-700 dark:text-gray-300">
											月度
										</span>
									</label>
									<label className="flex items-center">
										<input
											type="radio"
											checked={trendType === "week"}
											onChange={() => setTrendType("week")}
											className="mr-2 rounded border-gray-300 text-green-600 focus:ring-green-500"
										/>
										<span className="text-sm text-gray-700 dark:text-gray-300">
											周度
										</span>
									</label>
									<label className="flex items-center">
										<input
											type="radio"
											checked={trendType === "day"}
											onChange={() => setTrendType("day")}
											className="mr-2 rounded border-gray-300 text-green-600 focus:ring-green-500"
										/>
										<span className="text-sm text-gray-700 dark:text-gray-300">
											日度
										</span>
									</label>
								</div>
								{/* 收入/支出切换 */}
								<div className="flex items-center space-x-4 mt-2">
									<label className="flex items-center">
										<input
											type="checkbox"
											checked={showIncome}
											onChange={(e) => setShowIncome(e.target.checked)}
											className="mr-2 rounded border-gray-300 text-green-600 focus:ring-green-500"
										/>
										<span className="text-sm text-gray-700 dark:text-gray-300">
											显示收入
										</span>
									</label>
									<label className="flex items-center">
										<input
											type="checkbox"
											checked={showExpense}
											onChange={(e) => setShowExpense(e.target.checked)}
											className="mr-2 rounded border-gray-300 text-red-600 focus:ring-red-500"
										/>
										<span className="text-sm text-gray-700 dark:text-gray-300">
											显示支出
										</span>
									</label>
								</div>
							</div>
						</div>

						{/* 趋势图表 */}
						{trendLoading ? (
							<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6">
								<div className="flex items-center justify-center h-64">
									<div className="text-center">
										<div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-2" />
										<p className="text-gray-500 dark:text-gray-400">
											加载中...
										</p>
									</div>
								</div>
							</div>
						) : trendError ? (
							<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6">
								<div className="flex items-center justify-center h-64">
									<div className="text-center">
										<div className="text-red-500 text-lg mb-2">⚠️</div>
										<p className="text-red-600 dark:text-red-400">
											{trendError}
										</p>
										<button
											onClick={fetchTrendData}
											className="mt-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
										>
											重试
										</button>
									</div>
								</div>
							</div>
						) : (
							<FinancialTrendChart
								data={trendData}
								showIncome={showIncome}
								showExpense={showExpense}
								granularity={trendType}
								formatAmount={formatAmount}
							/>
						)}
					</div>

					{/* 统计期间 */}
					<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6">
						<h4 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
							统计期间
						</h4>
						<p className="text-gray-600 dark:text-gray-400">
							{financialStats.period_start} 至 {financialStats.period_end}
						</p>
					</div>
				</div>
			)}
		</div>
	);
};

export default StatsTab;
