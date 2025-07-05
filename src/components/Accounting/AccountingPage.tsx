import { invoke } from "@tauri-apps/api/core";
import type React from "react";
import { useCallback, useEffect, useRef, useState } from "react";
import type {
	AccountDto,
	AccountType,
	CreateAccountRequest,
	CreateTransactionRequest,
	FinancialStatsDto,
	TransactionDto,
	TransactionType,
} from "../../types";
import AccountsTab from "./AccountsTab";
import OverviewTab from "./OverviewTab";
import StatsTab from "./StatsTab";
import TransactionsTab from "./TransactionsTab";

const AccountingPage: React.FC = () => {
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
			amount: "",
			description: "",
			account_id: "",
			category_id: undefined,
			to_account_id: undefined,
			transaction_date: new Date().toISOString().split("T")[0],
			tags: [],
			receipt_path: undefined,
		});

	// 金额输入框ref
	const amountInputRef = useRef<HTMLInputElement>(null);

	// 当前正在编辑的交易（为空表示创建模式）
	const [editingTransaction, setEditingTransaction] =
		useState<TransactionDto | null>(null);

	// 弹窗打开时自动聚焦并全选金额输入框
	useEffect(() => {
		if (isCreateTransactionOpen && amountInputRef.current) {
			amountInputRef.current.focus();
			amountInputRef.current.select();
		}
	}, [isCreateTransactionOpen]);

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

	// 保存（创建/编辑）交易
	const saveTransaction = async () => {
		try {
			if (editingTransaction) {
				// 调用更新接口
				await invoke("update_transaction", {
					transactionId: editingTransaction.id,
					request: {
						transaction_type: newTransaction.transaction_type,
						amount: Number(newTransaction.amount) || 0,
						description: newTransaction.description,
						account_id: newTransaction.account_id,
						category_id: newTransaction.category_id,
						to_account_id: newTransaction.to_account_id,
						transaction_date: newTransaction.transaction_date,
						tags: newTransaction.tags,
						receipt_path: newTransaction.receipt_path,
					},
				});
			} else {
				// 调用创建接口
				await invoke("create_transaction", {
					request: {
						...newTransaction,
						amount: Number(newTransaction.amount) || 0,
					},
				});
			}
			setIsCreateTransactionOpen(false);
			setEditingTransaction(null);
			setNewTransaction({
				transaction_type: "expense",
				amount: "",
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
			console.error("保存交易失败:", err);
			setError("保存交易失败");
		}
	};

	// 打开编辑交易弹窗并预填数据
	const handleEditTransaction = (tx: TransactionDto) => {
		setEditingTransaction(tx);
		setNewTransaction({
			transaction_type: tx.transaction_type,
			amount: tx.amount.toString(),
			description: tx.description,
			account_id: tx.account_id,
			category_id: tx.category_id,
			to_account_id: tx.to_account_id,
			transaction_date: tx.transaction_date,
			tags: tx.tags,
			receipt_path: tx.receipt_path,
		});
		setIsCreateTransactionOpen(true);
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

	const renderActiveTab = () => {
		switch (activeTab) {
			case "overview":
				return (
					<OverviewTab
						accounts={accounts}
						financialStats={financialStats}
						transactions={transactions}
						formatAmount={formatAmount}
					/>
				);
			case "accounts":
				return (
					<AccountsTab
						accounts={accounts}
						formatAmount={formatAmount}
						getAccountTypeLabel={getAccountTypeLabel}
						onOpenCreateAccount={() => setIsCreateAccountOpen(true)}
					/>
				);
			case "transactions":
				return (
					<TransactionsTab
						transactions={transactions}
						formatAmount={formatAmount}
						getTransactionTypeLabel={getTransactionTypeLabel}
						onOpenCreateTransaction={() => setIsCreateTransactionOpen(true)}
						onEditTransaction={handleEditTransaction}
					/>
				);
			case "stats":
				return (
					<StatsTab
						financialStats={financialStats}
						formatAmount={formatAmount}
					/>
				);
			default:
				return null;
		}
	};

	return (
		<div className="flex flex-col h-full">
			{/* 内容主体（保留标签页等） */}
			<div className="flex flex-col px-2 h-full">
				{/* 标签与内容区域保留原结构，但移除多余边距 */}
				<div className="flex-shrink-0 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 overflow-x-auto sticky top-0 z-10 pt-2 md:pt-4">
					<div className="flex">
						{[
							{ key: "overview", label: "总览概览" },
							{ key: "accounts", label: "账户管理" },
							{ key: "transactions", label: "交易明细" },
							{ key: "stats", label: "统计分析" },
						].map((tab) => (
							<button
								key={tab.key}
								onClick={() =>
									setActiveTab(
										tab.key as
											| "overview"
											| "accounts"
											| "transactions"
											| "stats",
									)
								}
								className={`px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap border-b-2 ${
									activeTab === tab.key
										? "text-theme-primary border-theme-primary"
										: "text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 border-transparent"
								}`}
							>
								{tab.label}
							</button>
						))}
					</div>
				</div>

				{/* 错误提示 */}
				{error && (
					<div className="mx-6 mt-4 p-4 bg-red-100 dark:bg-red-900 border border-red-300 dark:border-red-700 rounded-lg">
						<p className="text-red-700 dark:text-red-300">{error}</p>
					</div>
				)}

				{/* 内容区域 - 可滚动 */}
				<div className="flex-1 overflow-y-auto py-4">{renderActiveTab()}</div>

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
								{editingTransaction ? "编辑交易" : "创建交易"}
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
										ref={amountInputRef}
										value={newTransaction.amount}
										onChange={(e) =>
											setNewTransaction({
												...newTransaction,
												amount: e.target.value,
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
									onClick={() => {
										setIsCreateTransactionOpen(false);
										setEditingTransaction(null);
									}}
									className="px-4 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
								>
									取消
								</button>
								<button
									onClick={saveTransaction}
									className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
								>
									{editingTransaction ? "保存" : "创建"}
								</button>
							</div>
						</div>
					</div>
				)}
			</div>
		</div>
	);
};

export default AccountingPage;
