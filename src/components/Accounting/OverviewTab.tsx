import type React from "react";
import type {
	AccountDto,
	FinancialStatsDto,
	TransactionDto,
} from "../../types";

interface OverviewTabProps {
	accounts: AccountDto[];
	financialStats: FinancialStatsDto | null;
	transactions: TransactionDto[];
	formatAmount: (amount: number, currency?: string) => string;
}

const OverviewTab: React.FC<OverviewTabProps> = ({
	accounts,
	financialStats,
	transactions,
	formatAmount,
}) => {
	return (
		<div className="space-y-6">
			<div className="flex justify-between items-center mb-2">
				<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
					总览概览
				</h3>
			</div>
			{/* 统计卡片区 */}
			<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
				{/* 总余额 */}
				<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6">
					<div className="flex items-center">
						<div className="flex-shrink-0 text-3xl">💰</div>
						<div className="ml-4">
							<p className="text-sm font-medium text-gray-500 dark:text-gray-400">
								总余额
							</p>
							<p className="text-2xl font-semibold text-gray-900 dark:text-white">
								{formatAmount(
									accounts.reduce((sum, acc) => sum + acc.balance, 0),
								)}
							</p>
						</div>
					</div>
				</div>
				{/* 本月收入 */}
				<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6">
					<div className="flex items-center">
						<div className="flex-shrink-0 text-3xl">📈</div>
						<div className="ml-4">
							<p className="text-sm font-medium text-gray-500 dark:text-gray-400">
								本月收入
							</p>
							<p className="text-2xl font-semibold text-gray-900 dark:text-white">
								{financialStats
									? formatAmount(financialStats.total_income)
									: "￥0.00"}
							</p>
						</div>
					</div>
				</div>
				{/* 本月支出 */}
				<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6">
					<div className="flex items-center">
						<div className="flex-shrink-0 text-3xl">📉</div>
						<div className="ml-4">
							<p className="text-sm font-medium text-gray-500 dark:text-gray-400">
								本月支出
							</p>
							<p className="text-2xl font-semibold text-gray-900 dark:text-white">
								{financialStats
									? formatAmount(financialStats.total_expense)
									: "￥0.00"}
							</p>
						</div>
					</div>
				</div>
				{/* 净收入 */}
				<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6">
					<div className="flex items-center">
						<div className="flex-shrink-0 text-3xl">💎</div>
						<div className="ml-4">
							<p className="text-sm font-medium text-gray-500 dark:text-gray-400">
								净收入
							</p>
							<p className="text-2xl font-semibold text-gray-900 dark:text-white">
								{financialStats
									? formatAmount(financialStats.net_income)
									: "￥0.00"}
							</p>
						</div>
					</div>
				</div>
			</div>
			{/* 最近交易卡片 */}
			<div className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-6">
				<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
					最近交易
				</h3>
				<div className="space-y-4">
					{transactions.slice(0, 5).map((transaction) => (
						<div
							key={transaction.id}
							className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-lg"
						>
							<div className="flex items-center space-x-4">
								<div
									className={`w-3 h-3 rounded-full ${
										transaction.transaction_type === "income"
											? "bg-green-500"
											: transaction.transaction_type === "expense"
												? "bg-red-500"
												: "bg-blue-500"
									}`}
								/>
								<div>
									<p className="font-medium text-gray-900 dark:text-gray-100">
										{transaction.description}
									</p>
									<p className="text-sm text-gray-500 dark:text-gray-400">
										{transaction.account_name} • {transaction.transaction_date}
									</p>
								</div>
							</div>
							<div
								className={`text-lg font-semibold ${
									transaction.transaction_type === "income"
										? "text-green-600 dark:text-green-400"
										: transaction.transaction_type === "expense"
											? "text-red-600 dark:text-red-400"
											: "text-blue-600 dark:text-blue-400"
								}`}
							>
								{transaction.transaction_type === "income"
									? "+"
									: transaction.transaction_type === "expense"
										? "-"
										: ""}
								{formatAmount(transaction.amount, transaction.currency)}
							</div>
						</div>
					))}
				</div>
			</div>
		</div>
	);
};

export default OverviewTab;
