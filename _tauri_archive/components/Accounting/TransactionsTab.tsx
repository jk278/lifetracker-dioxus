import type { TransactionDto } from "../../types";

interface TransactionsTabProps {
	transactions: TransactionDto[];
	formatAmount: (amount: number, currency?: string) => string;
	getTransactionTypeLabel: (type: string) => string;
	onOpenCreateTransaction: () => void;
	onEditTransaction: (tx: TransactionDto) => void;
}

const TransactionsTab: React.FC<TransactionsTabProps> = ({
	transactions,
	formatAmount,
	getTransactionTypeLabel,
	onOpenCreateTransaction,
	onEditTransaction,
}) => {
	return (
		<div className="space-y-6">
			<div className="flex justify-between items-center">
				<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
					交易记录
				</h3>
				<button
					onClick={onOpenCreateTransaction}
					className="px-4 py-2 bg-theme-primary text-white rounded-lg bg-theme-primary-hover theme-transition"
				>
					添加交易
				</button>
			</div>

			{/* 大屏表格布局 */}
			<div className="hidden md:block bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 overflow-hidden">
				<div className="overflow-x-auto">
					<table className="w-full">
						<thead className="bg-gray-50 dark:bg-gray-800">
							<tr>
								<th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
									类型
								</th>
								<th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
									描述
								</th>
								<th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
									账户
								</th>
								<th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
									金额
								</th>
								<th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
									日期
								</th>
								<th className="px-6 py-3 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider text-right">
									操作
								</th>
							</tr>
						</thead>
						<tbody className="divide-y divide-gray-200 dark:divide-gray-700">
							{transactions.map((transaction) => (
								<tr
									key={transaction.id}
									className="hover:bg-gray-50 dark:hover:bg-gray-800"
								>
									<td className="px-6 py-4 whitespace-nowrap">
										<span
											className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
												transaction.transaction_type === "income"
													? "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
													: transaction.transaction_type === "expense"
														? "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200"
														: "bg-theme-primary-light dark:bg-theme-primary-dark text-theme-primary-dark dark:text-theme-primary-lighter"
											}`}
										>
											{getTransactionTypeLabel(transaction.transaction_type)}
										</span>
									</td>
									<td className="px-6 py-4">
										<div className="text-sm font-medium text-gray-900 dark:text-gray-100">
											{transaction.description}
										</div>
									</td>
									<td className="px-6 py-4">
										<div className="text-sm text-gray-900 dark:text-gray-100">
											{transaction.account_name}
										</div>
										{transaction.to_account_name && (
											<div className="text-xs text-gray-500 dark:text-gray-400">
												→ {transaction.to_account_name}
											</div>
										)}
									</td>
									<td className="px-6 py-4">
										<div
											className={`text-sm font-medium ${
												transaction.transaction_type === "income"
													? "text-green-600 dark:text-green-400"
													: transaction.transaction_type === "expense"
														? "text-red-600 dark:text-red-400"
														: "text-theme-primary text-theme-primary-hover"
											}`}
										>
											{transaction.transaction_type === "income"
												? "+"
												: transaction.transaction_type === "expense"
													? "-"
													: ""}
											{formatAmount(transaction.amount, transaction.currency)}
										</div>
									</td>
									<td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">
										{transaction.transaction_date}
									</td>
									<td className="px-6 py-4 whitespace-nowrap text-right">
										<button
											onClick={() => onEditTransaction(transaction)}
											className="text-theme-primary text-theme-primary-hover text-sm theme-transition"
										>
											编辑
										</button>
									</td>
								</tr>
							))}
						</tbody>
					</table>
				</div>
			</div>

			{/* 小屏卡片布局 */}
			<div className="md:hidden space-y-4">
				{transactions.map((transaction) => (
					<div
						key={transaction.id}
						className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg dark:shadow-gray-700/20 p-4"
					>
						{/* 第一行：类型标签 + 金额 + 编辑按钮 */}
						<div className="flex justify-between items-center mb-2">
							<div className="flex items-center space-x-3">
								<span
									className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
										transaction.transaction_type === "income"
											? "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
											: transaction.transaction_type === "expense"
												? "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200"
												: "bg-theme-primary-light dark:bg-theme-primary-dark text-theme-primary-dark dark:text-theme-primary-lighter"
									}`}
								>
									{getTransactionTypeLabel(transaction.transaction_type)}
								</span>
								<div
									className={`text-lg font-semibold ${
										transaction.transaction_type === "income"
											? "text-green-600 dark:text-green-400"
											: transaction.transaction_type === "expense"
												? "text-red-600 dark:text-red-400"
												: "text-theme-primary text-theme-primary-hover"
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
							<button
								onClick={() => onEditTransaction(transaction)}
								className="text-theme-primary text-theme-primary-hover text-sm theme-transition px-3 py-1 rounded-md hover:bg-theme-primary-light dark:hover:bg-theme-primary-dark"
							>
								编辑
							</button>
						</div>
						
						{/* 第二行：描述 + 账户信息 + 日期 */}
						<div className="flex justify-between items-center text-sm">
							<div className="flex items-center space-x-2 flex-1 min-w-0">
								<span className="font-medium text-gray-900 dark:text-gray-100 truncate">
									{transaction.description}
								</span>
								<span className="text-gray-500 dark:text-gray-400">•</span>
								<span className="text-gray-700 dark:text-gray-300">
									{transaction.account_name}
								</span>
								{transaction.to_account_name && (
									<span className="text-gray-500 dark:text-gray-400">
										→ {transaction.to_account_name}
									</span>
								)}
							</div>
							<div className="text-xs text-gray-500 dark:text-gray-400 ml-2 flex-shrink-0">
								{transaction.transaction_date}
							</div>
						</div>
					</div>
				))}
			</div>

			{/* 无数据状态 */}
			{transactions.length === 0 && (
				<div className="text-center py-12">
					<div className="text-gray-500 dark:text-gray-400">
						暂无交易记录
					</div>
				</div>
			)}
		</div>
	);
};

export default TransactionsTab;
