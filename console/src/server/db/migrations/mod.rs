use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use logging::log;
use rbatis::RBatis;

/// 迁移执行方式
pub enum MigrationAction {
    /// 直接执行SQL（适用于 CREATE TABLE IF NOT EXISTS 等）
    #[allow(unused)]
    Sql(&'static str),
    /// 闭包（适用于需要条件判断的操作，如 ALTER TABLE ADD COLUMN）
    #[allow(unused)]
    Rust(fn(&RBatis) -> std::pin::Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + '_>>),
}

/// 数据库迁移定义
pub struct Migration {
    /// 版本号（必须递增，从1开始）
    pub version: i32,
    /// 迁移描述
    pub description: &'static str,
    /// 迁移动作
    pub action: MigrationAction,
}

impl Migration {
    /// 获取所有迁移（按版本号排序）
    ///
    /// 新增迁移时在此添加，版本号递增即可。
    /// 迁移SQL必须幂等：
    /// - 建表用 CREATE TABLE IF NOT EXISTS
    /// - 加列用 Rust 闭包先 PRAGMA 检查再 ALTER TABLE（对于Sqlite）
    /// - 建索引用 CREATE INDEX IF NOT EXISTS
    pub fn all() -> Vec<Migration> {
        let mut migrations: Vec<Migration> = vec![];
        migrations.sort_by_key(|m| m.version);
        migrations
    }
}

/// 获取当前数据库版本号（-1表示尚未执行任何迁移）
async fn get_db_version() -> anyhow::Result<i32> {
    let value = SystemConfig::get::<Option<String>>(ConfigKey::DbVersion).await?;
    Ok(value.and_then(|v| v.parse::<i32>().ok()).unwrap_or(-1))
}

/// 设置数据库版本号
async fn set_db_version(version: i32) -> anyhow::Result<()> {
    SystemConfig::upsert(ConfigKey::DbVersion, &version.to_string()).await?;
    Ok(())
}

/// 执行所有待运行的迁移
///
/// 在应用启动时调用，自动检测当前数据库版本并按顺序执行缺失的迁移。
/// - 新安装：init.sql 已创建最新表结构，直接跳过
/// - 旧版本升级：按版本号顺序逐个执行迁移，每个迁移保证幂等
pub async fn run_all(rb: &RBatis) -> anyhow::Result<()> {
    let current_version = get_db_version().await?;
    let migrations = Migration::all();
    let latest_version = migrations.last().map(|m| m.version).unwrap_or(0);

    if current_version == -1 {
        log::info!(
            "fresh install, no migration needed, setting db-version to {}",
            latest_version
        );
        set_db_version(latest_version).await?;
        return Ok(());
    }
    log::info!(
        "database version check - current: {}, latest: {}",
        current_version,
        latest_version
    );

    if current_version >= latest_version {
        log::info!("database is up to date, no migration needed");
        return Ok(());
    }

    for migration in migrations {
        if migration.version > current_version {
            log::info!(
                "applying migration v{}: {}",
                migration.version,
                migration.description
            );

            match &migration.action {
                MigrationAction::Sql(sql) => {
                    rb.exec(*sql, vec![]).await?;
                }
                MigrationAction::Rust(func) => {
                    func(rb).await?;
                }
            }

            // 每完成一个迁移立即更新版本号
            set_db_version(migration.version).await?;

            log::info!("migration v{} completed", migration.version);
        }
    }

    log::info!(
        "all migrations applied, database version: {}",
        latest_version
    );
    Ok(())
}
