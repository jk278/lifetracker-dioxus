//! # 数据库连接管理模块
//!
//! 提供SQLite数据库的连接管理和基本操作

use crate::errors::{AppError, Result};
use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// 数据库连接池
///
/// 提供读写分离的数据库连接管理
#[derive(Debug)]
pub struct DatabaseConnection {
    /// 数据库文件路径
    database_path: String,
    /// 写连接（互斥）
    write_connection: Arc<Mutex<Connection>>,
}

impl DatabaseConnection {
    /// 创建新的数据库连接池
    pub fn new<P: AsRef<Path>>(database_path: P) -> Result<Self> {
        let path_str = database_path.as_ref().to_string_lossy().to_string();

        // 确保数据库父目录存在
        if let Some(parent) = database_path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::Storage(format!("无法创建数据库目录 {}: {}", parent.display(), e))
            })?;
        }

        // 创建写连接
        let write_conn = Connection::open(&database_path)?;

        // 配置数据库参数
        let _journal_mode: String =
            write_conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
        write_conn.busy_timeout(std::time::Duration::from_secs(30))?;
        write_conn.execute("PRAGMA foreign_keys=ON", [])?;
        write_conn.execute("PRAGMA synchronous=NORMAL", [])?;

        Ok(Self {
            database_path: path_str,
            write_connection: Arc::new(Mutex::new(write_conn)),
        })
    }

    /// 创建只读连接
    fn create_read_connection(&self) -> Result<Connection> {
        let conn = Connection::open(&self.database_path)?;
        // 只读连接配置
        let _journal_mode: String =
            conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
        conn.busy_timeout(std::time::Duration::from_secs(10))?;
        conn.execute("PRAGMA query_only=ON", [])?;
        Ok(conn)
    }

    /// 执行读操作（不需要锁）
    pub fn read<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let read_conn = self.create_read_connection()?;
        f(&read_conn)
    }

    /// 执行写操作（需要独占锁）
    pub fn write<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T>,
    {
        let mut write_conn = self
            .write_connection
            .lock()
            .map_err(|_| AppError::System("Failed to acquire write lock".to_string()))?;
        f(&mut *write_conn)
    }

    /// 执行SQL语句（写操作）
    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize> {
        self.write(|conn| Ok(conn.execute(sql, params)?))
    }

    /// 查询单行数据（读操作）
    pub fn query_row<T, F>(&self, sql: &str, params: &[&dyn rusqlite::ToSql], f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> SqliteResult<T>,
    {
        self.read(|conn| Ok(conn.query_row(sql, params, f)?))
    }

    /// 开始事务（写操作）
    pub fn begin_transaction(&self) -> Result<()> {
        self.write(|conn| {
            conn.execute("BEGIN TRANSACTION", [])?;
            Ok(())
        })
    }

    /// 提交事务（写操作）
    pub fn commit_transaction(&self) -> Result<()> {
        self.write(|conn| {
            conn.execute("COMMIT", [])?;
            Ok(())
        })
    }

    /// 回滚事务（写操作）
    pub fn rollback_transaction(&self) -> Result<()> {
        self.write(|conn| {
            conn.execute("ROLLBACK", [])?;
            Ok(())
        })
    }

    /// 获取写连接的引用（用于迁移等特殊操作）
    pub fn get_raw_connection(&self) -> Arc<Mutex<Connection>> {
        self.write_connection.clone()
    }
}
