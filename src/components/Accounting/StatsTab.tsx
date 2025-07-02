import type React from "react";
import type { FinancialStatsDto } from "../../types";

interface StatsTabProps {
	financialStats: FinancialStatsDto | null;
	formatAmount: (amount: number, currency?: string) => string;
}

const StatsTab: React.FC<StatsTabProps> = ({
	financialStats,
	formatAmount,
}) => {
	return (
		<div className="p-6 h-full overflow-y-auto">
			<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-6">
				财务统计
			</h3>

			{financialStats && (
				<div className="space-y-6">
					{/* 统计卡片 */}
					<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
						<div className="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
							<h4 className="text-sm font-medium text-gray-500 dark:text-gray-400">
								总收入
							</h4>
							<p className="text-2xl font-bold text-green-600 dark:text-green-400">
								{formatAmount(financialStats.total_income)}
							</p>
						</div>
						<div className="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
							<h4 className="text-sm font-medium text-gray-500 dark:text-gray-400">
								总支出
							</h4>
							<p className="text-2xl font-bold text-red-600 dark:text-red-400">
								{formatAmount(financialStats.total_expense)}
							</p>
						</div>
						<div className="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
							<h4 className="text-sm font-medium text-gray-500 dark:text-gray-400">
								净收入
							</h4>
							<p className="text-2xl font-bold text-blue-600 dark:text-blue-400">
								{formatAmount(financialStats.net_income)}
							</p>
						</div>
						<div className="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
							<h4 className="text-sm font-medium text-gray-500 dark:text-gray-400">
								交易笔数
							</h4>
							<p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
								{financialStats.transaction_count}
							</p>
						</div>
					</div>

					{/* 统计期间 */}
					<div className="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
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