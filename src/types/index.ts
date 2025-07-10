// 任务数据类型
export interface Task {
	id: string;
	name: string;
	description?: string;
	category_id?: string;
	category_name?: string;
	start_time?: string;
	end_time?: string;
	duration_seconds: number;
	is_active: boolean;
	tags: string[];
	created_at: string;
	updated_at: string;
}

// 分类数据类型
export interface Category {
	id: string;
	name: string;
	description?: string;
	color: string;
	icon?: string;
	is_active: boolean;
	task_count: number;
	total_duration_seconds: number;
	created_at: string;
	updated_at: string;
}

// 计时器状态类型
export interface TimerStatus {
	state: "running" | "paused" | "stopped";
	current_task_id?: string;
	current_task_name?: string;
	start_time?: string;
	pause_time?: string;
	elapsed_seconds: number;
	total_today_seconds: number;
}

// 时间记录类型
export interface TimeEntry {
	id: number;
	task_name: string;
	category_name?: string;
	start_time: string;
	end_time?: string;
	duration_seconds: number;
}

// 统计数据类型
export interface Statistics {
	today: {
		total_seconds: number;
		task_count: number;
		active_categories: number;
		most_productive_hour?: number;
	};
	this_week: {
		total_seconds: number;
		task_count: number;
		active_days: number;
		average_daily_seconds: number;
	};
	category_stats: {
		category_id: string;
		category_name: string;
		total_seconds: number;
		task_count: number;
		percentage: number;
	}[];
}

// 创建任务请求类型
export interface CreateTaskRequest {
	name: string;
	description?: string;
	category_id?: string;
	tags?: string[];
}

// 更新任务请求类型
export interface UpdateTaskRequest {
	name?: string;
	description?: string;
	category_id?: string;
	tags?: string[];
}

// 应用配置类型
export interface AppConfig {
	theme: "light" | "dark" | "system";
	language: string;
	notifications: {
		enabled: boolean;
		sound: boolean;
	};
	timer: {
		auto_start: boolean;
		reminder_interval: number;
	};
}

// ==================== 记账功能类型定义 ====================

// 交易类型枚举
export type TransactionType = "income" | "expense" | "transfer";

// 交易状态枚举
export type TransactionStatus = "pending" | "completed" | "cancelled";

// 账户类型枚举
export type AccountType =
	| "cash"
	| "bank"
	| "credit_card"
	| "investment"
	| "other";

// 预算周期枚举
export type BudgetPeriod = "daily" | "weekly" | "monthly" | "yearly";

// 账户数据类型
export interface Account {
	id: string;
	name: string;
	account_type: AccountType;
	currency: string;
	balance: number;
	initial_balance: number;
	description?: string;
	is_active: boolean;
	is_default: boolean;
	created_at: string;
	updated_at: string;
}

// 交易记录类型
export interface Transaction {
	id: string;
	transaction_type: TransactionType;
	amount: number;
	currency: string;
	description: string;
	account_id: string;
	account_name?: string;
	category_id?: string;
	category_name?: string;
	to_account_id?: string; // 转账目标账户
	to_account_name?: string;
	status: TransactionStatus;
	transaction_date: string;
	tags: string[];
	receipt_path?: string; // 收据文件路径
	created_at: string;
	updated_at: string;
}

// 交易分类类型
export interface TransactionCategory {
	id: string;
	name: string;
	type: TransactionType;
	description?: string;
	color: string;
	icon?: string;
	parent_id?: string; // 父分类ID（支持层级分类）
	is_active: boolean;
	created_at: string;
	updated_at: string;
}

// 预算类型
export interface Budget {
	id: string;
	name: string;
	category_id: string;
	category_name?: string;
	amount: number;
	currency: string;
	period: BudgetPeriod;
	start_date: string;
	end_date?: string;
	spent_amount: number;
	remaining_amount: number;
	is_active: boolean;
	created_at: string;
	updated_at: string;
}

// 财务统计类型
export interface FinancialStats {
	total_income: number;
	total_expense: number;
	net_income: number;
	account_balance: number;
	transaction_count: number;
	period_start: string;
	period_end: string;
	currency: string;
	category_breakdown: {
		category_id: string;
		category_name: string;
		amount: number;
		percentage: number;
		transaction_count: number;
	}[];
}

// 创建账户请求类型
export interface CreateAccountRequest {
	name: string;
	account_type: AccountType;
	currency?: string;
	initial_balance?: number;
	description?: string;
	is_default?: boolean;
}

// 更新账户请求类型
export interface UpdateAccountRequest {
	name?: string;
	account_type?: AccountType;
	currency?: string;
	description?: string;
	is_active?: boolean;
	is_default?: boolean;
}

// 创建交易请求类型
export interface CreateTransactionRequest {
	transaction_type: TransactionType;
	amount: string | number;
	description: string;
	account_id: string;
	category_id?: string;
	to_account_id?: string;
	transaction_date: string;
	tags: string[];
	receipt_path?: string;
}

// 更新交易请求类型
export interface UpdateTransactionRequest {
	transaction_type?: TransactionType;
	amount?: number;
	description?: string;
	account_id?: string;
	category_id?: string;
	to_account_id?: string;
	transaction_date?: string;
	tags?: string[];
	receipt_path?: string;
	status?: TransactionStatus;
}

// 创建交易分类请求类型
export interface CreateTransactionCategoryRequest {
	name: string;
	type: TransactionType;
	description?: string;
	color: string;
	icon?: string;
	parent_id?: string;
}

// 更新交易分类请求类型
export interface UpdateTransactionCategoryRequest {
	name?: string;
	type?: TransactionType;
	description?: string;
	color?: string;
	icon?: string;
	parent_id?: string;
	is_active?: boolean;
}

// 创建预算请求类型
export interface CreateBudgetRequest {
	name: string;
	category_id: string;
	amount: number;
	currency: string;
	period: BudgetPeriod;
	start_date: string;
	end_date?: string;
}

// 更新预算请求类型
export interface UpdateBudgetRequest {
	name?: string;
	category_id?: string;
	amount?: number;
	currency?: string;
	period?: BudgetPeriod;
	start_date?: string;
	end_date?: string;
	is_active?: boolean;
}

// 交易查询参数类型
export interface TransactionQueryRequest {
	account_id?: string;
	category_id?: string;
	transaction_type?: TransactionType;
	status?: TransactionStatus;
	start_date?: string;
	end_date?: string;
	min_amount?: number;
	max_amount?: number;
	tags?: string[];
	search?: string;
	page?: number;
	limit?: number;
}

// 财务报表类型
export interface FinancialReport {
	period: {
		start_date: string;
		end_date: string;
	};
	summary: FinancialStats;
	income_breakdown: {
		category_id: string;
		category_name: string;
		amount: number;
		percentage: number;
		transactions: Transaction[];
	}[];
	expense_breakdown: {
		category_id: string;
		category_name: string;
		amount: number;
		percentage: number;
		transactions: Transaction[];
	}[];
	monthly_trend: MonthlyTrend[];
	account_balances: {
		account_id: string;
		account_name: string;
		balance: number;
		currency: string;
	}[];
}

// 月度趋势数据类型
export interface MonthlyTrend {
	month: string;
	income: number;
	expense: number;
	net: number;
}

// === 新增统一趋势类型 ===
export type TrendGranularity = "day" | "week" | "month";

export interface TrendData {
	/**
	 * X 轴标签，根据粒度可能是日期(YYYY-MM-DD)、周(YYYY-Www)或月(YYYY-MM)
	 */
	label: string;
	income: number;
	expense: number;
	net: number;
}

// ==================== DTO 类型别名 ====================
// 这些类型别名用于前后端通信，与 Rust 端的 DTO 类型保持一致

export type AccountDto = Account;
export type TransactionDto = Transaction;
export type TransactionCategoryDto = TransactionCategory;
export type BudgetDto = Budget;
export type FinancialStatsDto = FinancialStats;

// ==================== 笔记功能类型定义 ====================

// 心情枚举
export type MoodType = "happy" | "sad" | "neutral" | "excited" | "stressed" | "relaxed" | "anxious" | "confident";

// 笔记类型
export interface Note {
	id: string;
	title: string;
	content: string;
	mood?: MoodType;
	tags: string[];
	is_favorite: boolean;
	is_archived: boolean;
	created_at: string;
	updated_at: string;
}

// 创建笔记请求类型
export interface CreateNoteRequest {
	title: string;
	content: string;
	mood?: MoodType;
	tags?: string[];
	is_favorite?: boolean;
}

// 更新笔记请求类型
export interface UpdateNoteRequest {
	title?: string;
	content?: string;
	mood?: MoodType;
	tags?: string[];
	is_favorite?: boolean;
	is_archived?: boolean;
}

// 笔记查询参数类型
export interface NotesQueryRequest {
	search?: string;
	tags?: string[];
	mood?: MoodType;
	is_favorite?: boolean;
	is_archived?: boolean;
	start_date?: string;
	end_date?: string;
	limit?: number;
	offset?: number;
}

// 笔记统计类型
export interface NotesStats {
	total_notes: number;
	favorite_notes: number;
	archived_notes: number;
	notes_this_week: number;
	notes_this_month: number;
	most_used_tags: {
		tag: string;
		count: number;
	}[];
	mood_distribution: {
		mood: MoodType;
		count: number;
		percentage: number;
	}[];
	daily_notes_trend: {
		date: string;
		count: number;
	}[];
}

// 笔记 DTO 类型别名
export type NoteDto = Note;
export type NotesStatsDto = NotesStats;
