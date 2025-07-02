import type React from "react";
import type { AccountDto } from "../../types";

interface AccountsTabProps {
	accounts: AccountDto[];
	formatAmount: (amount: number, currency?: string) => string;
	getAccountTypeLabel: (type: string) => string;
	onOpenCreateAccount: () => void;
}

const AccountsTab: React.FC<AccountsTabProps> = ({
	accounts,
	formatAmount,
	getAccountTypeLabel,
	onOpenCreateAccount,
}) => {
	return (
		<div className="p-6 h-full overflow-y-auto">
			<div className="flex justify-between items-center mb-6">
				<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
					账户管理
				</h3>
				<button
					onClick={onOpenCreateAccount}
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
	);
};

export default AccountsTab; 