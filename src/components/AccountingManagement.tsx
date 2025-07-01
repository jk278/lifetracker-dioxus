// AccountingManagement.tsx - 记账功能管理组件

import { invoke } from "@tauri-apps/api/core";
import type React from "react";
import { useCallback, useEffect, useState } from "react";
import type {
	AccountDto,
	AccountType,
	CreateAccountRequest,
	CreateTransactionRequest,
	FinancialStatsDto,
	TransactionDto,
	TransactionType,
} from "../types";

const AccountingManagement: React.FC = () => {
	// 状态管理
	const [activeTab, setActiveTab] = useState<
		"overview" | "accounts" | "transactions" | "stats"
	>("overview");
	const [accounts, setAccounts] = useState<AccountDto[]>([]);
	const [transactions, setTransactions] = useState<TransactionDto[]>([]);
	const [financialStats, setFinancialStats] =
		useState<FinancialStatsDto | null>(null);
	const [_loading, setLoading] = useState(false);
	const [error, setError] = useState<string | null>(null);

	// 表单状态
	const [isCreateAccountOpen, setIsCreateAccountOpen] = useState(false);
	const [isCreateTransactionOpen, setIsCreateTransactionOpen] = useState(false);
	const [newAccount, setNewAccount] = useState<CreateAccountRequest>({
		name: "",
		account_type: "cash",
		currency: "CNY",
		initial_balance: 0,
		description: undefined,
		is_default: false,
	});
	const [newTransaction, setNewTransaction] =
		useState<CreateTransactionRequest>({
			transaction_type: "expense",
			amount: 0,
			description: "",
			account_id: "",
			category_id: undefined,
			to_account_id: undefined,
			transaction_date: new Date().toISOString().split("T")[0],
			tags: [],
			receipt_path: undefined,
		});

	// 获取数据的方法
	const fetchAccounts = useCallback(async () => {
		try {
			setLoading(true);
			const accountsData = await invoke<AccountDto[]>("get_accounts");
			setAccounts(accountsData);
			setError(null);
		} catch (err) {
			console.error("获取账户失败:", err);
			setError("获取账户失败");
		} finally {
			setLoading(false);
		}
	}, []);

	const fetchTransactions = useCallback(async () => {
		try {
			setLoading(true);
			const transactionsData =
				await invoke<TransactionDto[]>("get_transactions");
			setTransactions(transactionsData);
			setError(null);
		} catch (err) {
			console.error("获取交易记录失败:", err);
			setError("获取交易记录失败");
		} finally {
			setLoading(false);
		}
	}, []);

	const fetchFinancialStats = useCallback(async () => {
		try {
			const today = new Date();
			const startOfMonth = new Date(today.getFullYear(), today.getMonth(), 1);
			const endOfMonth = new Date(today.getFullYear(), today.getMonth() + 1, 0);

			const statsData = await invoke<FinancialStatsDto>("get_financial_stats", {
				startDate: startOfMonth.toISOString().split("T")[0],
				endDate: endOfMonth.toISOString().split("T")[0],
			});
			setFinancialStats(statsData);
			setError(null);
		} catch (err) {
			console.error("获取财务统计失败:", err);
			setError("获取财务统计失败");
		}
	}, []);

	// 创建账户
	const createAccount = async () => {
		try {
			await invoke("create_account", { request: newAccount });
			setIsCreateAccountOpen(false);
			setNewAccount({
				name: "",
				account_type: "cash",
				currency: "CNY",
				initial_balance: 0,
				description: undefined,
				is_default: false,
			});
			await fetchAccounts();
		} catch (err) {
			console.error("创建账户失败:", err);
			setError("创建账户失败");
		}
	};

	// 创建交易
	const createTransaction = async () => {
		try {
			await invoke("create_transaction", { request: newTransaction });
			setIsCreateTransactionOpen(false);
			setNewTransaction({
				transaction_type: "expense",
				amount: 0,
				description: "",
				account_id: "",
				category_id: undefined,
				to_account_id: undefined,
				transaction_date: new Date().toISOString().split("T")[0],
				tags: [],
				receipt_path: undefined,
			});
			await fetchTransactions();
			await fetchAccounts(); // 刷新账户余额
			await fetchFinancialStats(); // 刷新统计
		} catch (err) {
			console.error("创建交易失败:", err);
			setError("创建交易失败");
		}
	};

	// 初始化数据
	useEffect(() => {
		fetchAccounts();
		fetchTransactions();
		fetchFinancialStats();
	}, [fetchAccounts, fetchTransactions, fetchFinancialStats]);

	// 格式化金额
	const formatAmount = (amount: number, currency = "CNY") => {
		return new Intl.NumberFormat("zh-CN", {
			style: "currency",
			currency: currency,
			minimumFractionDigits: 2,
		}).format(amount);
	};

	// 获取账户类型显示名称
	const getAccountTypeLabel = (type: string) => {
		const types: Record<string, string> = {
			cash: "现金",
			bank: "银行卡",
			creditcard: "信用卡",
			investment: "投资账户",
			other: "其他",
		};
		return types[type] || type;
	};

	// 获取交易类型显示名称
	const getTransactionTypeLabel = (type: string) => {
		const types: Record<string, string> = {
			income: "收入",
			expense: "支出",
			transfer: "转账",
		};
		return types[type] || type;
	};

	return (
		<div className="space-y-6">
			{/* 页面标题 */}
			<div className="flex items-center justify-between">
				<h2 className="text-2xl font-bold text-gray-900 dark:text-white">
					记账管理
				</h2>
			</div>

			{/* 内容主体（保留标签页等） */}
			<div className="surface-adaptive rounded-lg shadow-lg dark:shadow-gray-700/20 flex flex-col h-[80vh]">
				{/* 标签与内容区域保留原结构，但移除多余边距 */}
				<div className="flex border-b border-gray-200 dark:border-gray-700 px-6">
					{[
						{ key: "overview", label: "总览", icon: "📊" },
						{ key: "accounts", label: "账户", icon: "🏦" },
						{ key: "transactions", label: "交易", icon: "💳" },
						{ key: "stats", label: "统计", icon: "📈" },
					].map((tab) => (
						<button
							key={tab.key}
							onClick={() => setActiveTab(tab.key as any)}
							className={`flex items-center space-x-2 px-4 py-3 border-b-2 font-medium text-sm transition-colors ${
								activeTab === tab.key
									? "border-blue-500 text-blue-600 dark:text-blue-400"
									: "border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 hover:border-gray-300"
							}`}
						>
							<span>{tab.icon}</span>
							<span>{tab.label}</span>
						</button>
					))}
				</div>

				{/* 错误提示 */}
				{error && (
					<div className="mx-6 mt-4 p-4 bg-red-100 dark:bg-red-900 border border-red-300 dark:border-red-700 rounded-lg">
						<p className="text-red-700 dark:text-red-300">{error}</p>
					</div>
				)}

				{/* 内容区域 */}
				<div className="flex-1 overflow-hidden">
					{/* 总览标签页 */}
					{activeTab === "overview" && (
						<div className="p-6 h-full overflow-y-auto">
							<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
								{/* 总余额 */}
								<div className="surface-adaptive rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
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
								<div className="surface-adaptive rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
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
								<div className="surface-adaptive rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
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
								<div className="surface-adaptive rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
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

							{/* 最近交易 */}
							<div className="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
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
														{transaction.account_name} •{" "}
														{transaction.transaction_date}
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
					)}

					{/* 账户标签页 */}
					{activeTab === "accounts" && (
						<div className="p-6 h-full overflow-y-auto">
							<div className="flex justify-between items-center mb-6">
								<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
									账户管理
								</h3>
								<button
									onClick={() => setIsCreateAccountOpen(true)}
									className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
								>
									添加账户
								</button>
							</div>

							<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
								{accounts.map((account) => (
									<div
										key={account.id}
										className="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
									>
										<div className="flex items-center justify-between mb-4">
											<h4 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
												{account.name}
											</h4>
											{account.is_default && (
												<span className="px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-xs rounded-full">
													默认
												</span>
											)}
										</div>
										<p className="text-sm text-gray-500 dark:text-gray-400 mb-2">
											{getAccountTypeLabel(account.account_type)}
										</p>
										<p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
											{formatAmount(account.balance, account.currency)}
										</p>
										{account.description && (
											<p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
												{account.description}
											</p>
										)}
									</div>
								))}
							</div>
						</div>
					)}

					{/* 交易标签页 */}
					{activeTab === "transactions" && (
						<div className="p-6 h-full overflow-y-auto">
							<div className="flex justify-between items-center mb-6">
								<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
									交易记录
								</h3>
								<button
									onClick={() => setIsCreateTransactionOpen(true)}
									className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
								>
									添加交易
								</button>
							</div>

							<div className="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
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
																		: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200"
															}`}
														>
															{getTransactionTypeLabel(
																transaction.transaction_type,
															)}
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
																		: "text-blue-600 dark:text-blue-400"
															}`}
														>
															{transaction.transaction_type === "income"
																? "+"
																: transaction.transaction_type === "expense"
																	? "-"
																	: ""}
															{formatAmount(
																transaction.amount,
																transaction.currency,
															)}
														</div>
													</td>
													<td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">
														{transaction.transaction_date}
													</td>
												</tr>
											))}
										</tbody>
									</table>
								</div>
							</div>
						</div>
					)}

					{/* 统计标签页 */}
					{activeTab === "stats" && (
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
											{financialStats.period_start} 至{" "}
											{financialStats.period_end}
										</p>
									</div>
								</div>
							)}
						</div>
					)}
				</div>

				{/* 创建账户弹窗 */}
				{isCreateAccountOpen && (
					<div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
						<div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md p-6">
							<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
								创建账户
							</h3>

							<div className="space-y-4">
								<div>
									<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
										账户名称
									</label>
									<input
										type="text"
										value={newAccount.name}
										onChange={(e) =>
											setNewAccount({ ...newAccount, name: e.target.value })
										}
										className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-100"
										placeholder="输入账户名称"
									/>
								</div>

								<div>
									<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
										账户类型
									</label>
									<select
										value={newAccount.account_type}
										onChange={(e) =>
											setNewAccount({
												...newAccount,
												account_type: e.target.value as AccountType,
											})
										}
										className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-100"
									>
										<option value="cash">现金</option>
										<option value="bank">银行卡</option>
										<option value="creditcard">信用卡</option>
										<option value="investment">投资账户</option>
										<option value="other">其他</option>
									</select>
								</div>

								<div>
									<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
										初始余额
									</label>
									<input
										type="number"
										step="0.01"
										value={newAccount.initial_balance}
										onChange={(e) =>
											setNewAccount({
												...newAccount,
												initial_balance: Number(e.target.value),
											})
										}
										className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-100"
										placeholder="0.00"
									/>
								</div>

								<div>
									<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
										描述（可选）
									</label>
									<textarea
										value={newAccount.description}
										onChange={(e) =>
											setNewAccount({
												...newAccount,
												description: e.target.value,
											})
										}
										className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-100"
										rows={3}
										placeholder="账户描述"
									/>
								</div>

								<div className="flex items-center">
									<input
										type="checkbox"
										id="isDefault"
										checked={newAccount.is_default}
										onChange={(e) =>
											setNewAccount({
												...newAccount,
												is_default: e.target.checked,
											})
										}
										className="mr-2"
									/>
									<label
										htmlFor="isDefault"
										className="text-sm text-gray-700 dark:text-gray-300"
									>
										设为默认账户
									</label>
								</div>
							</div>

							<div className="flex justify-end space-x-3 mt-6">
								<button
									onClick={() => setIsCreateAccountOpen(false)}
									className="px-4 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
								>
									取消
								</button>
								<button
									onClick={createAccount}
									className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
								>
									创建
								</button>
							</div>
						</div>
					</div>
				)}

				{/* 创建交易弹窗 */}
				{isCreateTransactionOpen && (
					<div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
						<div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md p-6">
							<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
								创建交易
							</h3>

							<div className="space-y-4">
								<div>
									<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
										交易类型
									</label>
									<select
										value={newTransaction.transaction_type}
										onChange={(e) =>
											setNewTransaction({
												...newTransaction,
												transaction_type: e.target.value as TransactionType,
											})
										}
										className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-100"
									>
										<option value="income">收入</option>
										<option value="expense">支出</option>
										<option value="transfer">转账</option>
									</select>
								</div>

								<div>
									<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
										金额
									</label>
									<input
										type="number"
										step="0.01"
										value={newTransaction.amount}
										onChange={(e) =>
											setNewTransaction({
												...newTransaction,
												amount: Number(e.target.value),
											})
										}
										className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-100"
										placeholder="0.00"
									/>
								</div>

								<div>
									<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
										描述
									</label>
									<input
										type="text"
										value={newTransaction.description}
										onChange={(e) =>
											setNewTransaction({
												...newTransaction,
												description: e.target.value,
											})
										}
										className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-100"
										placeholder="交易描述"
									/>
								</div>

								<div>
									<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
										账户
									</label>
									<select
										value={newTransaction.account_id}
										onChange={(e) =>
											setNewTransaction({
												...newTransaction,
												account_id: e.target.value,
											})
										}
										className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-100"
									>
										<option value="">选择账户</option>
										{accounts.map((account) => (
											<option key={account.id} value={account.id}>
												{account.name} (
												{formatAmount(account.balance, account.currency)})
											</option>
										))}
									</select>
								</div>

								{newTransaction.transaction_type === "transfer" && (
									<div>
										<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
											目标账户
										</label>
										<select
											value={newTransaction.to_account_id || ""}
											onChange={(e) =>
												setNewTransaction({
													...newTransaction,
													to_account_id: e.target.value
														? e.target.value
														: undefined,
												})
											}
											className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-100"
										>
											<option value="">选择目标账户</option>
											{accounts
												.filter((acc) => acc.id !== newTransaction.account_id)
												.map((account) => (
													<option key={account.id} value={account.id}>
														{account.name} (
														{formatAmount(account.balance, account.currency)})
													</option>
												))}
										</select>
									</div>
								)}

								<div>
									<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
										日期
									</label>
									<input
										type="date"
										value={newTransaction.transaction_date}
										onChange={(e) =>
											setNewTransaction({
												...newTransaction,
												transaction_date: e.target.value,
											})
										}
										className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-100"
									/>
								</div>
							</div>

							<div className="flex justify-end space-x-3 mt-6">
								<button
									onClick={() => setIsCreateTransactionOpen(false)}
									className="px-4 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
								>
									取消
								</button>
								<button
									onClick={createTransaction}
									className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
								>
									创建
								</button>
							</div>
						</div>
					</div>
				)}
			</div>
		</div>
	);
};

export default AccountingManagement;
