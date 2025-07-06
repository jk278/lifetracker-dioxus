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
		<div className="space-y-6">
			<div className="flex justify-between items-center">
				<h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
					账户管理
				</h3>
				<button
					onClick={onOpenCreateAccount}
					className="px-4 py-2 bg-theme-primary text-white rounded-lg bg-theme-primary-hover theme-transition"
				>
					添加账户
				</button>
			</div>

			<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
				{accounts.map((account) => (
					<div
						key={account.id}
						className="bg-surface rounded-lg border border-gray-200 dark:border-gray-700 p-6"
					>
						<div className="flex items-center justify-between mb-4">
							<h4 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
								{account.name}
							</h4>
							{account.is_default && (
								<span className="px-2 py-1 bg-theme-primary-light dark:bg-theme-primary-dark text-theme-primary-dark dark:text-theme-primary-lighter text-xs rounded-full">
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
