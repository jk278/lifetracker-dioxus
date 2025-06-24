import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    BarChart3,
    PieChart,
    Clock,
    Target,
    Activity
} from 'lucide-react';

interface StatisticsProps { }

// 匹配后端的StatisticsDto结构
interface StatisticsData {
    today: {
        date: string;
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
    this_month: {
        total_seconds: number;
        task_count: number;
        active_days: number;
        average_daily_seconds: number;
    };
    all_time: {
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
    daily_trend: {
        date: string;
        total_seconds: number;
        task_count: number;
    }[];
}

const Statistics: React.FC<StatisticsProps> = () => {
    const [period, setPeriod] = useState<string>('week');
    const [stats, setStats] = useState<StatisticsData | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const fetchStatistics = async () => {
        setLoading(true);
        setError(null);
        try {
            console.log('获取统计数据，周期:', period);
            // 调用后端的get_statistics命令，参数匹配后端实现
            const statisticsData = await invoke<StatisticsData>('get_statistics', {
                period: period
            });
            console.log('获取到统计数据:', statisticsData);
            setStats(statisticsData);
        } catch (error) {
            console.error('获取统计数据失败:', error);
            setError(`获取统计数据失败: ${error}`);
        } finally {
            setLoading(false);
        }
    };

    const formatDuration = (seconds: number): string => {
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        if (hours > 0) {
            return `${hours}h ${minutes}m`;
        }
        return `${minutes}m`;
    };

    const formatTime = (seconds: number): string => {
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        const secs = seconds % 60;
        return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    };

    useEffect(() => {
        fetchStatistics();
    }, [period]);

    if (error) {
        return (
            <div className="space-y-6">
                <h2 className="text-2xl font-bold text-gray-900 dark:text-white">统计报告</h2>
                <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
                    <p className="text-red-600 dark:text-red-400">{error}</p>
                    <button
                        onClick={fetchStatistics}
                        className="mt-2 px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
                    >
                        重试
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div className="space-y-6">
            {/* 页面标题和控制 */}
            <div className="flex items-center justify-between">
                <h2 className="text-2xl font-bold text-gray-900 dark:text-white">统计报告</h2>
                <div className="flex space-x-2">
                    <select
                        value={period}
                        onChange={(e) => setPeriod(e.target.value)}
                        className="px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
                    >
                        <option value="today">今日</option>
                        <option value="week">本周</option>
                        <option value="month">本月</option>
                        <option value="all">全部</option>
                    </select>
                </div>
            </div>

            {loading ? (
                <div className="flex justify-center py-12">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                </div>
            ) : stats ? (
                <>
                    {/* 总体统计卡片 */}
                    <div className="grid grid-cols-1 lg:grid-cols-4 md:grid-cols-2 gap-6">
                        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
                            <div className="flex items-center">
                                <div className="flex-shrink-0">
                                    <Clock className="h-8 w-8 text-blue-600 dark:text-blue-400" />
                                </div>
                                <div className="ml-4">
                                    <p className="text-sm font-medium text-gray-500 dark:text-gray-400">
                                        {period === 'today' ? '今日' : period === 'week' ? '本周' : period === 'month' ? '本月' : '总计'}时间
                                    </p>
                                    <p className="text-2xl font-semibold text-gray-900 dark:text-white">
                                        {formatTime(
                                            period === 'today' ? stats.today.total_seconds :
                                                period === 'week' ? stats.this_week.total_seconds :
                                                    period === 'month' ? stats.this_month.total_seconds :
                                                        stats.all_time.total_seconds
                                        )}
                                    </p>
                                </div>
                            </div>
                        </div>

                        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
                            <div className="flex items-center">
                                <div className="flex-shrink-0">
                                    <Target className="h-8 w-8 text-green-600 dark:text-green-400" />
                                </div>
                                <div className="ml-4">
                                    <p className="text-sm font-medium text-gray-500 dark:text-gray-400">任务数量</p>
                                    <p className="text-2xl font-semibold text-gray-900 dark:text-white">
                                        {period === 'today' ? stats.today.task_count :
                                            period === 'week' ? stats.this_week.task_count :
                                                period === 'month' ? stats.this_month.task_count :
                                                    stats.all_time.task_count}
                                    </p>
                                </div>
                            </div>
                        </div>

                        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
                            <div className="flex items-center">
                                <div className="flex-shrink-0">
                                    <BarChart3 className="h-8 w-8 text-purple-600 dark:text-purple-400" />
                                </div>
                                <div className="ml-4">
                                    <p className="text-sm font-medium text-gray-500 dark:text-gray-400">日均时间</p>
                                    <p className="text-2xl font-semibold text-gray-900 dark:text-white">
                                        {formatDuration(
                                            period === 'today' ? stats.today.total_seconds :
                                                period === 'week' ? stats.this_week.average_daily_seconds :
                                                    period === 'month' ? stats.this_month.average_daily_seconds :
                                                        stats.all_time.average_daily_seconds
                                        )}
                                    </p>
                                </div>
                            </div>
                        </div>

                        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
                            <div className="flex items-center">
                                <div className="flex-shrink-0">
                                    <Activity className="h-8 w-8 text-orange-600 dark:text-orange-400" />
                                </div>
                                <div className="ml-4">
                                    <p className="text-sm font-medium text-gray-500 dark:text-gray-400">活跃天数</p>
                                    <p className="text-2xl font-semibold text-gray-900 dark:text-white">
                                        {period === 'today' ? '1' :
                                            period === 'week' ? stats.this_week.active_days :
                                                period === 'month' ? stats.this_month.active_days :
                                                    stats.all_time.active_days}
                                    </p>
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* 分类统计 */}
                    {stats.category_stats && stats.category_stats.length > 0 && (
                        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
                            <div className="flex items-center justify-between mb-4">
                                <h3 className="text-lg font-semibold text-gray-900 dark:text-white">分类时间分布</h3>
                                <PieChart className="h-5 w-5 text-gray-400 dark:text-gray-500" />
                            </div>

                            <div className="space-y-4">
                                {stats.category_stats.map((category, index) => (
                                    <div key={index} className="flex items-center justify-between">
                                        <div className="flex items-center space-x-3">
                                            <div className="w-4 h-4 rounded-full bg-blue-600"></div>
                                            <span className="text-sm font-medium text-gray-900 dark:text-white">
                                                {category.category_name || '未分类'}
                                            </span>
                                        </div>
                                        <div className="text-right">
                                            <div className="text-sm font-semibold text-gray-900 dark:text-white">
                                                {formatDuration(category.total_seconds)}
                                            </div>
                                            <div className="text-xs text-gray-500 dark:text-gray-400">
                                                {category.percentage.toFixed(1)}% · {category.task_count}个任务
                                            </div>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}

                    {/* 提示信息 */}
                    <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
                        <p className="text-blue-600 dark:text-blue-400 text-sm">
                            💡 这是演示数据。完整的统计功能正在开发中，将基于您的实际工作记录生成详细报告。
                        </p>
                    </div>
                </>
            ) : (
                <div className="text-center py-12">
                    <p className="text-gray-500 dark:text-gray-400">暂无统计数据</p>
                </div>
            )}
        </div>
    );
};

export default Statistics; 