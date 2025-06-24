//! # 数据库迁移模块
//!
//! 管理数据库表结构的创建、更新和版本控制

use crate::errors::{AppError, Result};
use log::{debug, info, warn};
use rusqlite::Connection;

/// 数据库版本
const CURRENT_DB_VERSION: i32 = 1;

/// 迁移管理器
///
/// 负责管理数据库表结构的创建和更新
pub struct MigrationManager {
    /// 数据库连接
    connection: Connection,
}

impl MigrationManager {
    /// 创建新的迁移管理器
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    /// 运行所有必要的迁移
    ///
    /// 检查当前数据库版本，并运行所有必要的迁移脚本
    pub fn run_migrations(&mut self) -> Result<()> {
        info!("开始数据库迁移检查...");

        // 创建版本表（如果不存在）
        self.create_version_table()?;

        // 获取当前数据库版本
        let current_version = self.get_current_version()?;
        debug!("当前数据库版本: {}", current_version);

        // 运行必要的迁移
        for version in (current_version + 1)..=CURRENT_DB_VERSION {
            info!("运行迁移到版本 {}", version);
            self.run_migration_to_version(version)?;
            self.update_version(version)?;
        }

        info!("数据库迁移完成，当前版本: {}", CURRENT_DB_VERSION);
        Ok(())
    }

    /// 创建版本表
    fn create_version_table(&self) -> Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        "#;

        self.connection.execute(sql, [])?;
        debug!("版本表已创建或已存在");
        Ok(())
    }

    /// 获取当前数据库版本
    fn get_current_version(&self) -> Result<i32> {
        let result =
            self.connection
                .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                    row.get::<_, Option<i32>>(0)
                });

        match result {
            Ok(Some(version)) => Ok(version),
            Ok(None) => Ok(0), // 没有版本记录，从0开始
            Err(rusqlite::Error::SqliteFailure(_, Some(msg))) if msg.contains("no such table") => {
                Ok(0) // 版本表不存在，从0开始
            }
            Err(e) => Err(AppError::Database(e)),
        }
    }

    /// 更新版本号
    fn update_version(&self, version: i32) -> Result<()> {
        self.connection.execute(
            "INSERT INTO schema_version (version) VALUES (?1)",
            [version],
        )?;
        debug!("版本已更新到: {}", version);
        Ok(())
    }

    /// 运行指定版本的迁移
    fn run_migration_to_version(&self, version: i32) -> Result<()> {
        match version {
            1 => self.migration_v1(),
            _ => {
                warn!("未知的迁移版本: {}", version);
                Err(AppError::InvalidInput(format!(
                    "未知的迁移版本: {}",
                    version
                )))
            }
        }
    }

    /// 迁移到版本1：创建基础表
    fn migration_v1(&self) -> Result<()> {
        info!("运行迁移 v1: 创建基础表");

        // 开始事务
        let tx = self.connection.unchecked_transaction()?;

        // 创建分类表
        tx.execute(
            r#"
            CREATE TABLE IF NOT EXISTS categories (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                color TEXT NOT NULL DEFAULT '#2196F3',
                icon TEXT NOT NULL DEFAULT 'folder',
                daily_target_seconds INTEGER,
                weekly_target_seconds INTEGER,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                sort_order INTEGER NOT NULL DEFAULT 0,
                parent_id TEXT,
                created_at DATETIME NOT NULL,
                updated_at DATETIME,
                FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE SET NULL
            )
            "#,
            [],
        )?;

        // 创建时间记录表
        tx.execute(
            r#"
            CREATE TABLE IF NOT EXISTS time_entries (
                id TEXT PRIMARY KEY,
                task_name TEXT NOT NULL,
                category_id TEXT,
                start_time DATETIME NOT NULL,
                end_time DATETIME,
                duration_seconds INTEGER NOT NULL DEFAULT 0,
                description TEXT,
                tags TEXT NOT NULL DEFAULT '[]',
                created_at DATETIME NOT NULL,
                updated_at DATETIME,
                FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
            )
            "#,
            [],
        )?;

        // 创建任务表
        tx.execute(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                category_id TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                priority TEXT NOT NULL DEFAULT 'medium',
                estimated_duration_seconds INTEGER,
                total_duration_seconds INTEGER NOT NULL DEFAULT 0,
                tags TEXT NOT NULL DEFAULT '[]',
                due_date DATETIME,
                is_completed BOOLEAN NOT NULL DEFAULT 0,
                completed_at DATETIME,
                created_at DATETIME NOT NULL,
                updated_at DATETIME,
                FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
            )
            "#,
            [],
        )?;

        // 创建索引
        self.create_indexes(&tx)?;

        // 插入默认分类
        self.insert_default_categories(&tx)?;

        // 提交事务
        tx.commit()?;

        info!("迁移 v1 完成");
        Ok(())
    }

    /// 创建数据库索引
    fn create_indexes(&self, tx: &rusqlite::Transaction) -> Result<()> {
        debug!("创建数据库索引...");

        // 时间记录表索引
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_time_entries_start_time ON time_entries(start_time)",
            [],
        )?;

        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_time_entries_category_id ON time_entries(category_id)",
            [],
        )?;

        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_time_entries_task_name ON time_entries(task_name)",
            [],
        )?;

        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_time_entries_created_at ON time_entries(created_at)",
            [],
        )?;

        // 分类表索引
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_categories_parent_id ON categories(parent_id)",
            [],
        )?;

        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_categories_sort_order ON categories(sort_order)",
            [],
        )?;

        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_categories_is_active ON categories(is_active)",
            [],
        )?;

        // 任务表索引
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_category_id ON tasks(category_id)",
            [],
        )?;

        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)",
            [],
        )?;

        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at)",
            [],
        )?;

        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date)",
            [],
        )?;

        debug!("数据库索引创建完成");
        Ok(())
    }

    /// 插入默认分类
    fn insert_default_categories(&self, tx: &rusqlite::Transaction) -> Result<()> {
        debug!("插入默认分类...");

        let default_categories = vec![
            ("工作", "工作相关任务", "#FF5722", "work"),
            ("学习", "学习和培训", "#2196F3", "school"),
            ("个人", "个人事务", "#4CAF50", "person"),
            ("娱乐", "休闲娱乐", "#9C27B0", "games"),
            ("运动", "体育锻炼", "#FF9800", "fitness_center"),
            ("其他", "其他未分类任务", "#607D8B", "more_horiz"),
        ];

        for (i, (name, description, color, icon)) in default_categories.iter().enumerate() {
            let id = uuid::Uuid::new_v4().to_string();
            let created_at = chrono::Local::now().to_rfc3339();

            tx.execute(
                r#"
                INSERT OR IGNORE INTO categories (
                    id, name, description, color, icon, is_active, sort_order, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)
                "#,
                [
                    &id,
                    &name.to_string(),
                    &description.to_string(),
                    &color.to_string(),
                    &icon.to_string(),
                    &(i as i32).to_string(),
                    &created_at,
                ],
            )?;
        }

        debug!("默认分类插入完成");
        Ok(())
    }

    /// 重置数据库（删除所有表）
    ///
    /// **警告：这将删除所有数据！**
    pub fn reset_database(&self) -> Result<()> {
        warn!("重置数据库 - 这将删除所有数据！");

        let tables = vec!["time_entries", "categories", "tasks", "schema_version"];

        for table in tables {
            let sql = format!("DROP TABLE IF EXISTS {}", table);
            self.connection.execute(&sql, [])?;
            debug!("删除表: {}", table);
        }

        info!("数据库重置完成");
        Ok(())
    }

    /// 检查数据库完整性
    pub fn check_integrity(&self) -> Result<bool> {
        debug!("检查数据库完整性...");

        let result = self
            .connection
            .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))?;

        let is_ok = result == "ok";

        if is_ok {
            info!("数据库完整性检查通过");
        } else {
            warn!("数据库完整性检查失败: {}", result);
        }

        Ok(is_ok)
    }

    /// 优化数据库
    pub fn optimize_database(&self) -> Result<()> {
        info!("优化数据库...");

        // 分析查询计划
        self.connection.execute("ANALYZE", [])?;
        debug!("数据库分析完成");

        // 清理未使用的空间
        self.connection.execute("VACUUM", [])?;
        debug!("数据库清理完成");

        info!("数据库优化完成");
        Ok(())
    }

    /// 获取数据库统计信息
    pub fn get_database_stats(&self) -> Result<DatabaseStats> {
        debug!("获取数据库统计信息...");

        // 获取表大小信息
        let page_count: i64 = self
            .connection
            .query_row("PRAGMA page_count", [], |row| row.get(0))?;

        let page_size: i64 = self
            .connection
            .query_row("PRAGMA page_size", [], |row| row.get(0))?;

        let database_size = page_count * page_size;

        // 获取记录数量
        let time_entries_count: i64 = self
            .connection
            .query_row("SELECT COUNT(*) FROM time_entries", [], |row| row.get(0))
            .unwrap_or(0);

        let categories_count: i64 = self
            .connection
            .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
            .unwrap_or(0);

        let stats = DatabaseStats {
            database_size_bytes: database_size,
            time_entries_count,
            categories_count,
            current_version: self.get_current_version()?,
        };

        debug!("数据库统计信息: {:?}", stats);
        Ok(stats)
    }

    /// 备份数据库到指定路径
    pub fn backup_to_file<P: AsRef<std::path::Path>>(&self, backup_path: P) -> Result<()> {
        info!("备份数据库到: {:?}", backup_path.as_ref());

        // 使用SQLite的备份API
        let mut backup_conn = Connection::open(&backup_path)?;
        let backup = rusqlite::backup::Backup::new(&self.connection, &mut backup_conn)?;

        backup.run_to_completion(5, std::time::Duration::from_millis(250), None)?;

        info!("数据库备份完成");
        Ok(())
    }

    /// 从备份文件恢复数据库
    pub fn restore_from_file<P>(&mut self, backup_path: P) -> Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        info!("从备份恢复数据库: {:?}", backup_path.as_ref());

        let source_conn = Connection::open(&backup_path)?;
        let backup = rusqlite::backup::Backup::new(&source_conn, &mut self.connection)?;

        backup.run_to_completion(5, std::time::Duration::from_millis(250), None)?;

        info!("数据库恢复完成");
        Ok(())
    }
}

/// 数据库统计信息
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    /// 数据库文件大小（字节）
    pub database_size_bytes: i64,
    /// 时间记录数量
    pub time_entries_count: i64,
    /// 分类数量
    pub categories_count: i64,
    /// 当前数据库版本
    pub current_version: i32,
}

impl DatabaseStats {
    /// 获取格式化的数据库大小
    pub fn formatted_size(&self) -> String {
        let size = self.database_size_bytes as f64;

        if size >= 1024.0 * 1024.0 {
            format!("{:.2} MB", size / (1024.0 * 1024.0))
        } else if size >= 1024.0 {
            format!("{:.2} KB", size / 1024.0)
        } else {
            format!("{} bytes", size)
        }
    }
}

/// 便捷函数：运行迁移
///
/// 创建迁移管理器并运行所有必要的迁移
pub fn run_migrations(connection: Connection) -> Result<Connection> {
    let mut migration_manager = MigrationManager::new(connection);
    migration_manager.run_migrations()?;
    Ok(migration_manager.connection)
}

/// 便捷函数：初始化数据库
///
/// 创建数据库文件并运行所有迁移
pub fn initialize_database<P: AsRef<std::path::Path>>(database_path: P) -> Result<Connection> {
    info!("初始化数据库: {:?}", database_path.as_ref());

    // 确保父目录存在
    if let Some(parent) = database_path.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }

    let connection = Connection::open(&database_path)?;

    // 启用外键约束
    connection.execute("PRAGMA foreign_keys = ON", [])?;

    // 启用WAL模式
    let journal_mode: String =
        connection.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    info!("数据库日志模式设置为: {}", journal_mode);

    // 运行迁移
    run_migrations(connection)
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    fn create_test_database() -> (Connection, ::tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(db_path).unwrap();
        (conn, temp_dir)
    }

    #[test]
    fn test_migration_manager_creation() {
        let (conn, _temp_dir) = create_test_database();
        let _manager = MigrationManager::new(conn);
    }

    #[test]
    fn test_version_table_creation() {
        let (conn, _temp_dir) = create_test_database();
        let manager = MigrationManager::new(conn);

        assert!(manager.create_version_table().is_ok());

        // 检查版本表是否存在
        let result = manager.connection.query_row(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |row| row.get::<_, String>(0),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "schema_version");
    }

    #[test]
    fn test_get_current_version() {
        let (conn, _temp_dir) = create_test_database();
        let manager = MigrationManager::new(conn);

        // 初始版本应该是0
        assert_eq!(manager.get_current_version().unwrap(), 0);

        // 创建版本表后仍然是0
        manager.create_version_table().unwrap();
        assert_eq!(manager.get_current_version().unwrap(), 0);
    }

    #[test]
    fn test_update_version() {
        let (conn, _temp_dir) = create_test_database();
        let manager = MigrationManager::new(conn);

        manager.create_version_table().unwrap();
        manager.update_version(1).unwrap();

        assert_eq!(manager.get_current_version().unwrap(), 1);
    }

    #[test]
    fn test_run_migrations() {
        let (conn, _temp_dir) = create_test_database();
        let mut manager = MigrationManager::new(conn);

        assert!(manager.run_migrations().is_ok());

        // 检查表是否创建
        let tables = ["categories", "time_entries", "tasks", "schema_version"];

        for table in &tables {
            let result = manager.connection.query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get::<_, String>(0),
            );

            assert!(result.is_ok(), "表 {} 应该存在", table);
        }

        // 检查版本
        assert_eq!(manager.get_current_version().unwrap(), CURRENT_DB_VERSION);
    }

    #[test]
    fn test_database_stats() {
        let (conn, _temp_dir) = create_test_database();
        let mut manager = MigrationManager::new(conn);

        manager.run_migrations().unwrap();

        let stats = manager.get_database_stats().unwrap();

        assert!(stats.database_size_bytes > 0);
        assert!(stats.categories_count >= 6); // 默认分类数量
        assert_eq!(stats.time_entries_count, 0);
        assert_eq!(stats.current_version, CURRENT_DB_VERSION);
    }

    #[test]
    fn test_initialize_database() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("init_test.db");

        let conn = initialize_database(&db_path).unwrap();

        // 检查文件是否创建
        assert!(db_path.exists());

        // 检查表是否存在
        let result = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .unwrap();

        assert!(result >= 3); // 至少有3个表
    }

    #[test]
    fn test_database_integrity() {
        let (conn, _temp_dir) = create_test_database();
        let mut manager = MigrationManager::new(conn);

        manager.run_migrations().unwrap();

        let is_ok = manager.check_integrity().unwrap();
        assert!(is_ok);
    }

    #[test]
    fn test_backup_and_restore() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("original.db");
        let backup_path = temp_dir.path().join("backup.db");

        // 创建原始数据库
        let conn = initialize_database(&db_path).unwrap();
        let manager = MigrationManager::new(conn);

        // 备份
        assert!(manager.backup_to_file(&backup_path).is_ok());
        assert!(backup_path.exists());

        // 恢复到新数据库
        let restore_path = temp_dir.path().join("restored.db");
        let restore_conn = Connection::open(&restore_path).unwrap();
        let mut restore_manager = MigrationManager::new(restore_conn);

        assert!(restore_manager.restore_from_file(&backup_path).is_ok());

        // 验证恢复的数据库
        let stats = restore_manager.get_database_stats().unwrap();
        assert!(stats.categories_count >= 6);
    }
}
