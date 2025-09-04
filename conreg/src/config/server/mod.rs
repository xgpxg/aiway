use crate::Args;
use crate::raft::RaftRequest;
use anyhow::bail;
use chrono::{DateTime, Local};
use common::id;
use logging::log;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use std::fmt::Debug;
use std::time::Duration;

pub mod api;

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigEntry {
    /// 递增ID
    pub id_: i64,
    /// 命名空间
    pub namespace_id: String,
    /// 配置ID
    pub id: String,
    /// 配置内容
    pub content: String,
    /// 时间戳
    pub ts: DateTime<Local>,
    /// 描述
    pub description: Option<String>,
    /// md5
    pub md5: String,
}

impl ConfigEntry {
    pub fn gen_md5(content: &str) -> String {
        let digest = md5::compute(content);
        format!("{:x}", digest)
    }
}
/// 配置管理
#[derive(Debug)]
pub struct ConfigManager {
    /// 本地sqlite数据库，用于维护配置内容存储。
    /// 通过raft保证一致性
    pool: SqlitePool,
    /// Http客户端，主要用于同步log到集群
    http_client: reqwest::Client,
    /// 启动参数
    args: Args,
    /// 配置变化通知
    sender: tokio::sync::broadcast::Sender<String>,
}

impl ConfigManager {
    pub async fn new(args: &Args) -> anyhow::Result<Self> {
        let db_url = &format!("sqlite:{}/{}/{}", args.data_dir, "db", "config.db");
        log::info!("db url: {}", db_url);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;
        Self::init(&pool).await?;
        let network = reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(3))
            .read_timeout(Duration::from_secs(60))
            .build()?;

        let (sender, _) = tokio::sync::broadcast::channel(1024);
        Ok(Self {
            pool,
            http_client: network,
            args: args.clone(),
            sender,
        })
    }

    /// 初始化数据库
    async fn init(pool: &SqlitePool) -> anyhow::Result<()> {
        let sql = include_str!("../../db/init.sql");
        sqlx::query(sql).execute(pool).await?;
        Ok(())
    }

    fn notify_config_change(&self, namespace_id: String) {
        let _ = self.sender.send(namespace_id);
    }

    /// 获取配置
    pub async fn get_config(
        &self,
        namespace_id: &str,
        config_id: &str,
    ) -> anyhow::Result<Option<ConfigEntry>> {
        let config: Option<ConfigEntry> =
            sqlx::query_as("SELECT * FROM config WHERE namespace_id = ? AND id = ?")
                .bind(namespace_id)
                .bind(config_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(config)
    }

    /// 创建或更新配置，并同步到集群的其他节点
    pub async fn upsert_config_and_sync(
        &self,
        namespace_id: &str,
        config_id: &str,
        content: &str,
        description: Option<String>,
    ) -> anyhow::Result<()> {
        // 旧配置
        let config = self.get_config(namespace_id, config_id).await?;
        // 新配置的MD5
        let md5 = ConfigEntry::gen_md5(content);
        // 配置内容未改变，不处理
        if config.is_some() && config.as_ref().unwrap().md5 == md5 {
            log::info!("config content not change");
            return Ok(());
        }

        match config {
            None => {
                let entry = ConfigEntry {
                    id_: id::next(),
                    namespace_id: namespace_id.to_string(),
                    id: config_id.to_string(),
                    content: content.to_string(),
                    ts: Local::now(),
                    description,
                    md5,
                };
                // 同步数据
                self.sync(RaftRequest::SetConfig { entry }).await?;
            }
            Some(old) => {
                let entry = ConfigEntry {
                    id_: old.id_,
                    namespace_id: namespace_id.to_string(),
                    id: config_id.to_string(),
                    content: content.to_string(),
                    ts: Local::now(),
                    description,
                    md5,
                };
                // 同步数据
                self.sync(RaftRequest::UpdateConfig { entry }).await?;
            }
        }

        self.notify_config_change(namespace_id.to_string());

        Ok(())
    }

    /// 新增配置
    ///
    /// 注意：该方法不应该直接调用，而需要由raft apply log时调用，以保证数据一致性
    pub async fn insert_config(&self, entry: ConfigEntry) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO config (id_, namespace_id, id, content, description, ts, md5) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
            .bind(&entry.id_)
            .bind(&entry.namespace_id)
            .bind(&entry.id)
            .bind(&entry.content)
            .bind(&entry.description)
            .bind(&entry.ts)
            .bind(&entry.md5)
            .execute(&self.pool)
            .await?;

        // 添加历史记录
        self.append_history(&entry).await?;

        Ok(())
    }

    /// 更新配置
    ///
    /// 注意：该方法不应该直接调用，而需要由raft apply log时调用，以保证数据一致性
    pub async fn update_config(&self, entry: ConfigEntry) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE config SET content = ?, description = ?, ts = ?, md5 = ? WHERE id_ = ?",
        )
        .bind(&entry.content)
        .bind(&entry.description)
        .bind(&entry.ts)
        .bind(&entry.md5)
        .bind(&entry.id_)
        .execute(&self.pool)
        .await?;

        // 添加历史记录
        self.append_history(&entry).await?;

        Ok(())
    }

    /// 删除并同步到集群
    ///
    /// 不直接删除，提交命令到raft执行
    pub async fn delete_config_and_sync(
        &self,
        namespace_id: &str,
        config_id: &str,
    ) -> anyhow::Result<()> {
        self.sync(RaftRequest::DeleteConfig {
            namespace_id: namespace_id.to_string(),
            id: config_id.to_string(),
        })
        .await?;

        Ok(())
    }

    pub async fn delete_config(&self, namespace_id: &str, config_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM config WHERE namespace_id = ? AND id = ?")
            .bind(namespace_id)
            .bind(config_id)
            .execute(&self.pool)
            .await?;

        // 删除历史
        self.delete_history(namespace_id, config_id).await?;

        Ok(())
    }

    #[allow(unused)]
    pub async fn get_history(
        &self,
        namespace_id: &str,
        config_id: &str,
    ) -> anyhow::Result<Vec<ConfigEntry>> {
        let rows: Vec<ConfigEntry> = sqlx::query_as(
            "SELECT * FROM config_history WHERE namespace_id = ? AND id = ? ORDER BY id_ DESC",
        )
        .bind(namespace_id)
        .bind(config_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn append_history(&self, entry: &ConfigEntry) -> anyhow::Result<()> {
        log::info!("append history: {:?}", entry);
        // 保存历史
        sqlx::query(
            "INSERT INTO config_history (id_, namespace_id, id, content, description, ts, md5) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        // 注意这个ID，不能自增或随机生成，需要从entry中计算而来，以保证多节点下的数据的一致性
        .bind(&entry.id_ + entry.ts.timestamp_millis())
        .bind(&entry.namespace_id)
        .bind(&entry.id)
        .bind(&entry.content)
        .bind(&entry.description)
        .bind(&entry.ts)
        .bind(&entry.md5)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_history(&self, namespace_id: &str, id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM config_history WHERE namespace_id = ? AND id = ?")
            .bind(&namespace_id)
            .bind(&id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 恢复配置
    ///
    /// - id_: 配置历史ID
    pub async fn recovery(&self, id_: i64) -> anyhow::Result<()> {
        let history: Option<ConfigEntry> =
            sqlx::query_as("SELECT * FROM config_history WHERE id_ = ?")
                .bind(id_)
                .fetch_optional(&self.pool)
                .await?;

        if history.is_none() {
            bail!("No history config found with id {}", id_);
        }

        let history = history.unwrap();

        self.upsert_config_and_sync(
            &history.namespace_id,
            &history.id,
            &history.content,
            history.description,
        )
        .await?;

        Ok(())
    }

    /// 将配置变更提交到raft集群执行，使得raft应用变更日志，以保持数据一致性，
    /// 同步操作会阻塞进行，直到raft日志同步成功（即超过半数的节点写入成功）
    async fn sync(&self, request: RaftRequest) -> anyhow::Result<()> {
        log::info!("sync config request: {:?}", request);
        self.http_client
            .post(format!("http://127.0.0.1:{}/write", self.args.port))
            .json(&request)
            .send()
            .await?;
        log::info!("sync config success");
        Ok(())
    }

    /// 查询配置列表（分页）
    pub async fn list_configs_with_page(
        &self,
        namespace_id: &str,
        page_num: i32,
        page_size: i32,
    ) -> anyhow::Result<(u64, Vec<ConfigEntry>)> {
        let total: u64 = sqlx::query_scalar("SELECT COUNT(1) FROM config WHERE namespace_id = ?")
            .bind(namespace_id)
            .fetch_one(&self.pool)
            .await?;

        let offset = (page_num - 1) * page_size;

        let rows: Vec<ConfigEntry> = sqlx::query_as(
            "SELECT * FROM config WHERE namespace_id = ? ORDER BY id_ DESC LIMIT ?, ?",
        )
        .bind(namespace_id)
        .bind(offset)
        .bind(page_size)
        .fetch_all(&self.pool)
        .await?;

        Ok((total, rows))
    }

    /// 查询配置历史列表（分页）
    pub async fn list_config_history_with_page(
        &self,
        namespace_id: &str,
        id: &str,
        page_num: i32,
        page_size: i32,
    ) -> anyhow::Result<(u64, Vec<ConfigEntry>)> {
        let total: u64 = sqlx::query_scalar(
            "SELECT COUNT(1) FROM config_history WHERE namespace_id = ? AND id = ?",
        )
        .bind(namespace_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        let offset = (page_num - 1) * page_size;

        let rows: Vec<ConfigEntry> = sqlx::query_as(
            "SELECT * FROM config_history WHERE namespace_id = ? AND id = ? ORDER BY id_ DESC LIMIT ?, ?",
        )
        .bind(namespace_id)
        .bind(id)
        .bind(offset)
        .bind(page_size)
        .fetch_all(&self.pool)
        .await?;

        Ok((total, rows))
    }
}

#[derive(Debug)]
pub struct ConfigApp {
    pub manager: ConfigManager,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_config() {
        let args = Args {
            address: "127.0..0.1".to_string(),
            port: 8000,
            data_dir: "./data".to_string(),
            node_id: 1,
        };
        let cm = ConfigManager::new(&args).await.unwrap();
        let config = cm.get_config("public", "test").await.unwrap();
        println!("config: {:?}", config);

        let mut entry = ConfigEntry {
            id_: 1,
            namespace_id: "public".to_string(),
            id: "test".to_string(),
            content: "name: 0".to_string(),
            description: None,
            ts: Local::now(),
            md5: "".to_string(),
        };
        cm.insert_config(entry).await.unwrap();

        let config = cm.get_config("public", "test").await.unwrap();
        println!("config: {:?}", config);

        cm.update_config(entry).await.unwrap();

        let config = cm.get_config("public", "test").await.unwrap();
        println!("config: {:?}", config);

        let history = cm.get_history("public", "test").await.unwrap();
        println!("history: {:?}", history);

        cm.recovery(1).await.unwrap();
        let config = cm.get_config("public", "test").await.unwrap();
        println!("config: {:?}", config);
        let history = cm.get_history("public", "test").await.unwrap();
        println!("history: {:?}", history);
    }

    #[tokio::test]
    async fn test_id() {
        id::init();
        println!("{}", id::next());
    }
}
