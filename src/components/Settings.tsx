import { invoke } from "@tauri-apps/api/core";
import {
	Bell,
	Database,
	Download,
	Monitor,
	Moon,
	Palette,
	RefreshCw,
	Save,
	Settings as SettingsIcon,
	Sun,
	Upload,
} from "lucide-react";
import type React from "react";
import { useEffect, useState } from "react";
import { useTheme } from "../hooks/useTheme";

type SettingsProps = {};

interface AppConfig {
	theme: string;
	auto_save: boolean;
	notification_enabled: boolean;
	work_session_duration: number;
	break_duration: number;
	long_break_duration: number;
	data_retention_days: number;
	backup_enabled: boolean;
	language: string;
}

const Settings: React.FC<SettingsProps> = () => {
	const { theme, setTheme } = useTheme();
	const [config, setConfig] = useState<AppConfig>({
		theme: "system",
		auto_save: true,
		notification_enabled: true,
		work_session_duration: 25,
		break_duration: 5,
		long_break_duration: 15,
		data_retention_days: 365,
		backup_enabled: false,
		language: "zh-CN",
	});
	const [loading, setLoading] = useState(false);
	const [saving, setSaving] = useState(false);

	const fetchConfig = async () => {
		setLoading(true);
		try {
			const appConfig = await invoke<AppConfig>("get_config");
			setConfig(appConfig);
		} catch (error) {
			console.error("获取配置失败:", error);
		} finally {
			setLoading(false);
		}
	};

	const saveConfig = async () => {
		setSaving(true);
		try {
			await invoke("update_config", { config });
			alert("设置保存成功！");
		} catch (error) {
			console.error("保存配置失败:", error);
			alert("保存失败，请重试。");
		} finally {
			setSaving(false);
		}
	};

	const exportData = async () => {
		try {
			await invoke("export_data", {
				request: {
					format: "json",
					include_all: true,
				},
			});
			alert("数据导出成功！");
		} catch (error) {
			console.error("导出数据失败:", error);
			alert("导出失败，请重试。");
		}
	};

	const importData = async () => {
		if (!confirm("导入数据将覆盖现有数据，确定要继续吗？")) return;

		try {
			// 这里需要实现文件选择器，暂时使用假的路径
			await invoke("import_data", {
				file_path: "/path/to/import/file.json",
			});
			alert("数据导入成功！");
		} catch (error) {
			console.error("导入数据失败:", error);
			alert("导入失败，请重试。");
		}
	};

	const clearData = async () => {
		if (
			!confirm(
				"这将删除所有数据，包括任务、分类和计时记录。此操作不可恢复，确定要继续吗？",
			)
		)
			return;

		try {
			await invoke("clear_all_data");
			alert("数据已清除！");
		} catch (error) {
			console.error("清除数据失败:", error);
			alert("清除失败，请重试。");
		}
	};

	useEffect(() => {
		fetchConfig();
	}, []);

	if (loading) {
		return (
			<div className="flex justify-center py-12">
				<div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
			</div>
		);
	}

	return (
		<div className="space-y-6">
			{/* 页面标题 */}
			<div className="flex items-center justify-between">
				<h2 className="text-2xl font-bold text-gray-900 dark:text-white">
					设置
				</h2>
				<button
					onClick={saveConfig}
					disabled={saving}
					className="flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors"
				>
					<Save className="h-4 w-4 mr-2" />
					{saving ? "保存中..." : "保存设置"}
				</button>
			</div>

			<div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
				{/* 界面设置 */}
				<div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
					<div className="flex items-center mb-4">
						<Palette className="h-5 w-5 text-blue-600 dark:text-blue-400 mr-2" />
						<h3 className="text-lg font-semibold text-gray-900 dark:text-white">
							界面设置
						</h3>
					</div>

					<div className="space-y-4">
						<div>
							<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
								主题设置
							</label>
							<div className="grid grid-cols-3 gap-3">
								{[
									{ value: "system", label: "跟随系统", icon: Monitor },
									{ value: "light", label: "浅色", icon: Sun },
									{ value: "dark", label: "深色", icon: Moon },
								].map(({ value, label, icon: Icon }) => (
									<button
										key={value}
										onClick={() => setTheme(value as any)}
										className={`flex flex-col items-center p-3 rounded-lg border-2 transition-all ${
											theme === value
												? "border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300"
												: "border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:border-gray-300 dark:hover:border-gray-500"
										}`}
									>
										<Icon className="h-5 w-5 mb-1" />
										<span className="text-sm font-medium">{label}</span>
									</button>
								))}
							</div>
						</div>

						<div>
							<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
								语言
							</label>
							<select
								value={config.language}
								onChange={(e) =>
									setConfig({ ...config, language: e.target.value })
								}
								className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
							>
								<option value="zh-CN">简体中文</option>
								<option value="en-US">English</option>
							</select>
						</div>
					</div>
				</div>

				{/* 通知设置 */}
				<div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
					<div className="flex items-center mb-4">
						<Bell className="h-5 w-5 text-green-600 dark:text-green-400 mr-2" />
						<h3 className="text-lg font-semibold text-gray-900 dark:text-white">
							通知设置
						</h3>
					</div>

					<div className="space-y-4">
						<div className="flex items-center justify-between">
							<label className="text-sm font-medium text-gray-700 dark:text-gray-300">
								启用通知
							</label>
							<input
								type="checkbox"
								checked={config.notification_enabled}
								onChange={(e) =>
									setConfig({
										...config,
										notification_enabled: e.target.checked,
									})
								}
								className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 rounded"
							/>
						</div>

						<div className="flex items-center justify-between">
							<label className="text-sm font-medium text-gray-700 dark:text-gray-300">
								自动保存
							</label>
							<input
								type="checkbox"
								checked={config.auto_save}
								onChange={(e) =>
									setConfig({ ...config, auto_save: e.target.checked })
								}
								className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 rounded"
							/>
						</div>
					</div>
				</div>

				{/* 计时设置 */}
				<div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
					<div className="flex items-center mb-4">
						<SettingsIcon className="h-5 w-5 text-purple-600 dark:text-purple-400 mr-2" />
						<h3 className="text-lg font-semibold text-gray-900 dark:text-white">
							计时设置
						</h3>
					</div>

					<div className="space-y-4">
						<div>
							<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
								工作时长 (分钟)
							</label>
							<input
								type="number"
								min="1"
								max="120"
								value={config.work_session_duration}
								onChange={(e) =>
									setConfig({
										...config,
										work_session_duration: Number.parseInt(e.target.value),
									})
								}
								className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
							/>
						</div>

						<div>
							<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
								短休息时长 (分钟)
							</label>
							<input
								type="number"
								min="1"
								max="30"
								value={config.break_duration}
								onChange={(e) =>
									setConfig({
										...config,
										break_duration: Number.parseInt(e.target.value),
									})
								}
								className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
							/>
						</div>

						<div>
							<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
								长休息时长 (分钟)
							</label>
							<input
								type="number"
								min="1"
								max="60"
								value={config.long_break_duration}
								onChange={(e) =>
									setConfig({
										...config,
										long_break_duration: Number.parseInt(e.target.value),
									})
								}
								className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
							/>
						</div>
					</div>
				</div>

				{/* 数据管理 */}
				<div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg dark:shadow-gray-700/20 p-6">
					<div className="flex items-center mb-4">
						<Database className="h-5 w-5 text-orange-600 dark:text-orange-400 mr-2" />
						<h3 className="text-lg font-semibold text-gray-900 dark:text-white">
							数据管理
						</h3>
					</div>

					<div className="space-y-4">
						<div>
							<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
								数据保留天数
							</label>
							<input
								type="number"
								min="30"
								max="3650"
								value={config.data_retention_days}
								onChange={(e) =>
									setConfig({
										...config,
										data_retention_days: Number.parseInt(e.target.value),
									})
								}
								className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
							/>
							<p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
								超过此天数的数据将被自动删除
							</p>
						</div>

						<div className="flex items-center justify-between">
							<label className="text-sm font-medium text-gray-700 dark:text-gray-300">
								启用自动备份
							</label>
							<input
								type="checkbox"
								checked={config.backup_enabled}
								onChange={(e) =>
									setConfig({ ...config, backup_enabled: e.target.checked })
								}
								className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 rounded"
							/>
						</div>

						<div className="pt-4 border-t border-gray-200 dark:border-gray-600">
							<div className="flex flex-col space-y-2">
								<button
									onClick={exportData}
									className="flex items-center justify-center px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
								>
									<Download className="h-4 w-4 mr-2" />
									导出数据
								</button>

								<button
									onClick={importData}
									className="flex items-center justify-center px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
								>
									<Upload className="h-4 w-4 mr-2" />
									导入数据
								</button>

								<button
									onClick={clearData}
									className="flex items-center justify-center px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 transition-colors"
								>
									<RefreshCw className="h-4 w-4 mr-2" />
									清除所有数据
								</button>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
};

export default Settings;
