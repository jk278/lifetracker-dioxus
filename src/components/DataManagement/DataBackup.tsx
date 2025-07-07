import { invoke } from "@tauri-apps/api/core";
import { appDataDir, join, resolve as pathResolve } from "@tauri-apps/api/path";
import { open } from "@tauri-apps/plugin-dialog";
import {
	ArrowLeft,
	Database,
	Download,
	FolderOpen,
	Save,
	Settings,
	Upload,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useNavigation } from "../../hooks/useRouter";

// 获取应用数据目录
async function getAppDataDir(): Promise<string> {
	return await appDataDir();
}

export function DataBackup() {
	const { canGoBack, goBack } = useNavigation();

	const [backupPath, setBackupPath] = useState<string>("");

	// 备份设置本地状态
	const [backupSettings, setBackupSettings] = useState({
		autoBackup: true,
		backupInterval: 7,
		backupRetention: 30,
		backupDirectory: "",
	});

	// 初始化时加载配置中的备份设置，并确保目录为绝对路径
	useEffect(() => {
		(async () => {
			try {
				const cfg: any = await invoke("get_config");

				// 读取备份目录，如果是相对路径则转换为绝对路径（相对应用数据目录）
				let dir = cfg?.data?.backup_directory ?? "";
				const isAbsolute =
					typeof dir === "string" &&
					(dir.startsWith("/") || /^[a-zA-Z]:[\\/]/.test(dir));

				try {
					if (!isAbsolute && dir) {
						const base = await getAppDataDir();
						dir = await join(base, dir);
					}

					if (!dir) {
						const base = await getAppDataDir();
						dir = await join(base, "backups");
					}
				} catch (err) {
					console.warn("无法获取默认备份目录", err);
				}

				setBackupSettings({
					autoBackup: cfg?.data?.auto_backup ?? true,
					backupInterval: cfg?.data?.backup_interval ?? 7,
					backupRetention: cfg?.data?.backup_retention ?? 30,
					backupDirectory: dir,
				});
			} catch (e) {
				console.error("读取配置失败", e);
			}
		})();
	}, []);

	// 获取当前（或默认）备份目录
	const getEffectiveBackupDir = useCallback(async () => {
		let dir = backupSettings.backupDirectory;

		if (!dir) {
			const base = await getAppDataDir();
			dir = await join(base, "backups");
		}

		// 转换为绝对路径，确保文件对话框能够正确识别
		dir = await pathResolve(dir);

		return dir;
	}, [backupSettings.backupDirectory]);

	// 选择备份目录
	const chooseBackupDirectory = useCallback(async () => {
		try {
			const defaultDir = await getEffectiveBackupDir();
			const dir = await open({ directory: true, defaultPath: defaultDir });
			if (dir && !Array.isArray(dir)) {
				setBackupSettings((prev) => ({ ...prev, backupDirectory: dir }));
			}
		} catch (e) {
			console.error(e);
		}
	}, [getEffectiveBackupDir]);

	// 保存备份设置到配置
	const handleSaveBackupSettings = useCallback(async () => {
		try {
			const cfg: any = await invoke("get_config");

			// 保存前再次确保备份目录为绝对路径
			let dir = backupSettings.backupDirectory;
			const isAbsolute =
				typeof dir === "string" &&
				(dir.startsWith("/") || /^[a-zA-Z]:[\\/]/.test(dir));

			if (!isAbsolute) {
				const base = await getAppDataDir();
				dir = await join(base, dir);
			}

			const updated = {
				...cfg,
				data: {
					...cfg.data,
					auto_backup: backupSettings.autoBackup,
					backup_interval: backupSettings.backupInterval,
					backup_retention: backupSettings.backupRetention,
					backup_directory: dir,
				},
			};

			await invoke("update_config", { config: updated });
			alert("备份设置已保存！");
		} catch (e) {
			console.error(e);
			alert("保存失败，请重试");
		}
	}, [backupSettings]);

	// 立即备份
	const handleBackup = useCallback(async () => {
		try {
			if (!backupSettings.backupDirectory) {
				alert("请先在下方选择备份目录，再执行立即备份。");
				return;
			}
			const now = new Date();
			const timestamp = now
				.toISOString()
				.replace(/T/, "_")
				.replace(/:/g, "-")
				.split(".")[0]; // YYYY-MM-DD_HH-MM-SS
			const path = await join(
				backupSettings.backupDirectory,
				`lifetracker-backup-${timestamp}.db`,
			);
			setBackupPath(path);
			const res = await invoke<string>("backup_database", {
				destPath: path,
			});
			alert(res);
		} catch (e) {
			console.error(e);
			alert("备份失败");
		}
	}, [backupSettings.backupDirectory]);

	// 从备份恢复
	const handleRestore = useCallback(async () => {
		try {
			const defaultDir = await getEffectiveBackupDir();
			const filePath = await open({
				filters: [
					{ name: "SQLite Backup", extensions: ["db", "sqlite", "bak"] },
				],
				multiple: false,
				defaultPath: defaultDir,
			});
			if (!filePath || Array.isArray(filePath)) return;
			if (!confirm("导入备份将覆盖当前数据库，确定继续？")) return;
			const res = await invoke<string>("restore_database", {
				srcPath: filePath,
			});
			alert(res);
		} catch (e) {
			console.error(e);
			alert("恢复失败");
		}
	}, [getEffectiveBackupDir]);

	// 返回处理
	const handleBack = useCallback(() => {
		if (canGoBack) {
			goBack();
		}
	}, [canGoBack, goBack]);

	return (
		<div className="h-full flex flex-col">
			{/* 固定顶部导航栏 */}
			<div className="flex-shrink-0 px-4 md:px-6 py-4 border-b border-gray-200 dark:border-gray-700 surface-adaptive">
				<div className="flex items-center justify-between">
					<div className="flex items-center space-x-3">
						<button
							onClick={handleBack}
							className="flex items-center justify-center w-8 h-8 text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
							title="返回"
						>
							<ArrowLeft className="w-5 h-5" />
						</button>
						<h1 className="text-2xl font-bold text-gray-900 dark:text-white">
							备份与恢复
						</h1>
					</div>
				</div>
			</div>

			{/* 可滚动内容区域 */}
			<div className="flex-1 overflow-y-auto py-4 px-4 md:px-6 scroll-container">
				<div className="max-w-2xl mx-auto space-y-6">
					{/* 自动备份设置 */}
					<div className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6">
						<div className="flex items-center mb-4">
							<Settings className="h-5 w-5 text-blue-600 dark:text-blue-400 mr-2" />
							<h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
								自动备份设置
							</h2>
						</div>

						<div className="space-y-4">
							{/* 启用自动备份 */}
							<label className="flex items-center">
								<input
									type="checkbox"
									checked={backupSettings.autoBackup}
									onChange={(e) =>
										setBackupSettings((prev) => ({
											...prev,
											autoBackup: e.target.checked,
										}))
									}
									className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
								/>
								<span className="ml-2 text-sm text-gray-700 dark:text-gray-300">
									启用自动备份
								</span>
							</label>

							{/* 备份间隔和保留数量 */}
							<div className="grid grid-cols-2 gap-4">
								<div>
									<label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
										备份间隔（天）
									</label>
									<input
										type="number"
										min={1}
										value={backupSettings.backupInterval}
										onChange={(e) =>
											setBackupSettings((prev) => ({
												...prev,
												backupInterval: Number(e.target.value),
											}))
										}
										className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
									/>
								</div>
								<div>
									<label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
										保留备份数量
									</label>
									<input
										type="number"
										min={1}
										value={backupSettings.backupRetention}
										onChange={(e) =>
											setBackupSettings((prev) => ({
												...prev,
												backupRetention: Number(e.target.value),
											}))
										}
										className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
									/>
								</div>
							</div>

							{/* 备份目录选择 */}
							<div>
								<label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
									备份目录
								</label>
								<div className="flex items-center space-x-2">
									<input
										type="text"
										readOnly
										value={backupSettings.backupDirectory}
										className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-gray-50 dark:bg-gray-800 text-gray-500 dark:text-gray-400"
									/>
									<button
										onClick={chooseBackupDirectory}
										className="flex items-center px-3 py-2 bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 rounded-md text-sm transition-colors"
									>
										<FolderOpen className="h-4 w-4 mr-1" />
										选择
									</button>
								</div>
							</div>

							{/* 保存设置按钮 */}
							<button
								onClick={handleSaveBackupSettings}
								className="w-full flex items-center justify-center px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 transition-colors"
							>
								<Save className="h-4 w-4 mr-2" />
								保存备份设置
							</button>
						</div>
					</div>

					{/* 手动备份与恢复 */}
					<div className="surface-adaptive rounded-lg border border-gray-200 dark:border-gray-700 p-6">
						<div className="flex items-center mb-4">
							<Database className="h-5 w-5 text-green-600 dark:text-green-400 mr-2" />
							<h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
								手动操作
							</h2>
						</div>

						<div className="space-y-4">
							{/* 立即备份 */}
							<div className="flex flex-col sm:flex-row gap-4">
								<button
									onClick={handleBackup}
									className="flex-1 flex items-center justify-center px-4 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-colors"
								>
									<Download className="h-5 w-5 mr-2" />
									立即备份
								</button>

								<button
									onClick={handleRestore}
									className="flex-1 flex items-center justify-center px-4 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2 transition-colors"
								>
									<Upload className="h-5 w-5 mr-2" />
									从备份恢复
								</button>
							</div>

							{/* 最近备份文件 */}
							{backupPath && (
								<div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-3">
									<p className="text-xs text-gray-500 dark:text-gray-400 mb-1">
										最近备份文件:
									</p>
									<p className="text-sm text-gray-700 dark:text-gray-300 font-mono break-all">
										{backupPath}
									</p>
								</div>
							)}
						</div>
					</div>

					{/* 使用说明 */}
					<div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-6">
						<h3 className="text-lg font-semibold text-blue-900 dark:text-blue-100 mb-3">
							备份说明
						</h3>
						<div className="text-sm text-blue-800 dark:text-blue-200 space-y-2">
							<p>
								<strong>自动备份：</strong>应用会根据设置的间隔自动创建备份文件
							</p>
							<p>
								<strong>手动备份：</strong>立即创建一个备份文件到指定目录
							</p>
							<p>
								<strong>从备份恢复：</strong>
								选择备份文件并恢复数据（会覆盖当前数据）
							</p>
							<p>
								<strong>注意：</strong>恢复操作会覆盖当前所有数据，请谨慎操作
							</p>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
}
