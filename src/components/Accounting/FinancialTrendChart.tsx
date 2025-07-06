import {
	Bar,
	BarChart,
	CartesianGrid,
	Legend,
	ResponsiveContainer,
	Tooltip,
	XAxis,
	YAxis,
} from "recharts";
import { useEffect, useState } from "react";
import type { TrendData, TrendGranularity } from "../../types";

interface FinancialTrendChartProps {
	data: TrendData[];
	showIncome: boolean;
	showExpense: boolean;
	granularity: TrendGranularity;
	formatAmount: (amount: number, currency?: string) => string;
}

const FinancialTrendChart: React.FC<FinancialTrendChartProps> = ({
	data,
	showIncome,
	showExpense,
	granularity,
	formatAmount,
}) => {
	// 响应式检测
	const [isMobile, setIsMobile] = useState(false);
	
	useEffect(() => {
		const checkScreenSize = () => {
			if (typeof window !== 'undefined') {
				setIsMobile(window.innerWidth < 768);
			}
		};
		
		checkScreenSize();
		
		if (typeof window !== 'undefined') {
			window.addEventListener('resize', checkScreenSize);
			return () => window.removeEventListener('resize', checkScreenSize);
		}
	}, []);

	// 如果没有数据，显示空状态
	if (!data || data.length === 0) {
		return (
			<div className="flex items-center justify-center h-full bg-gray-50 dark:bg-gray-800 rounded-lg">
				<div className="text-center">
					<div className="text-gray-400 dark:text-gray-500 text-lg mb-2">
						📊
					</div>
					<p className="text-gray-500 dark:text-gray-400">暂无趋势数据</p>
				</div>
			</div>
		);
	}

	// 格式化月份显示（YYYY-MM -> MM月）
	const formatLabel = (label: string) => {
		if (granularity === "day") {
			return label.split("-").slice(1).join("-"); // MM-DD
		}
		if (granularity === "week") {
			return label.replace(/\d{4}-W/, "W"); // W27
		}
		// month
		return `${label.split("-")[1]}月`;
	};

	// 自定义工具提示
	const CustomTooltip = ({ active, payload, label }: any) => {
		if (active && payload && payload.length) {
			return (
				<div className="bg-surface border border-gray-200 dark:border-gray-600 rounded-lg shadow-lg p-4">
					<p className="font-medium text-gray-900 dark:text-gray-100 mb-2">
						{label}
					</p>
					{payload.map((entry: any) => (
						<p
							key={entry.name}
							className="text-sm"
							style={{ color: entry.color }}
						>
							{entry.name}: {formatAmount(entry.value)}
						</p>
					))}
				</div>
			);
		}
		return null;
	};

	// 根据屏幕宽度调整图表配置
	const chartMargin = isMobile 
		? { top: 5, right: 5, left: 5, bottom: 5 }
		: { top: 15, right: 15, left: 15, bottom: 15 };
	
	const tickFontSize = isMobile ? 9 : 12;
	const legendFontSize = isMobile ? 9 : 12;

	return (
		<div className="h-full">
			<ResponsiveContainer width="100%" height="100%">
				<BarChart
					data={data}
					margin={chartMargin}
					barCategoryGap={isMobile ? "10%" : "20%"}
				>
					<CartesianGrid strokeDasharray="3 3" className={isMobile ? "opacity-20" : "opacity-30"} />
					<XAxis
						dataKey="label"
						tickFormatter={formatLabel}
						className="text-gray-600 dark:text-gray-400"
						tick={{ fontSize: tickFontSize }}
						height={isMobile ? 20 : 30}
					/>
					<YAxis
						tickFormatter={(value) => Math.round(value).toString()}
						className="text-gray-600 dark:text-gray-400"
						tick={{ fontSize: tickFontSize }}
						width={isMobile ? 30 : 40}
					/>
					<Tooltip content={<CustomTooltip />} />
					<Legend 
						wrapperStyle={{ fontSize: `${legendFontSize}px` }}
						height={isMobile ? 20 : 30}
					/>

					{showIncome && (
						<Bar
							dataKey="income"
							name="收入"
							fill="#10b981"
							radius={[2, 2, 0, 0]}
						/>
					)}

					{showExpense && (
						<Bar
							dataKey="expense"
							name="支出"
							fill="#ef4444"
							radius={[2, 2, 0, 0]}
						/>
					)}
				</BarChart>
			</ResponsiveContainer>
		</div>
	);
};

export default FinancialTrendChart;
