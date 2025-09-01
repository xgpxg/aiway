use crate::Args;
use crate::raft::RaftRequest;
use chrono::{DateTime, Local};
use clap::Parser;
use logging::log;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use std::fmt::Debug;
use std::process::exit;
use std::time::Duration;

pub mod api;
mod res;

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigEntry {
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
}

impl ConfigEntry {
    pub fn gen_key(namespace_id: &str, id: &str) -> String {
        format!("{}:{}", namespace_id, id)
    }
}

/// 配置管理
#[derive(Debug, Clone)]
pub struct ConfigManager {
    /// 本地sqlite数据库，用于维护配置内容存储。
    /// 通过raft保证一致性
    pool: SqlitePool,
    network: reqwest::Client,
    port: u16,
}

/*impl Default for ConfigManager {
    fn default() -> Self {
        let args = Args::parse();
        Self::new(args.port,&format!("{}",args.data_dir))
    }
}*/

impl ConfigManager {
    pub async fn new(port: u16, db_url: &str) -> anyhow::Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;
        Self::init(&pool).await?;
        let network = reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(3))
            .read_timeout(Duration::from_secs(60))
            .build()?;
        Ok(Self {
            pool,
            network,
            port,
        })
    }

    async fn init(pool: &SqlitePool) -> anyhow::Result<()> {
        let sql = include_str!("init.sql");
        sqlx::query(sql).execute(pool).await?;
        Ok(())
    }
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

    pub async fn upsert_config_and_sync(
        &self,
        namespace_id: &str,
        config_id: &str,
        content: &str,
        description: Option<String>,
    ) -> anyhow::Result<()> {
        self.upsert_config(namespace_id, config_id, content, description)
            .await?;
        let config = self.get_config(namespace_id, config_id).await?;
        if config.is_none() {
            log::error!("config upsert, but config not found");
            return Ok(());
        }
        // 同步数据
        self.sync(RaftRequest::SetConfig {
            entry: config.unwrap(),
        })
        .await?;
        Ok(())
    }
    pub async fn upsert_config(
        &self,
        namespace_id: &str,
        config_id: &str,
        content: &str,
        description: Option<String>,
    ) -> anyhow::Result<()> {
        let config = self.get_config(namespace_id, config_id).await?;
        if config.is_none() {
            sqlx::query(
                "INSERT INTO config (namespace_id, id, content, description) VALUES (?, ?, ?, ?)",
            )
            .bind(namespace_id)
            .bind(config_id)
            .bind(content)
            .bind(description)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE config SET content = ?, description = ? WHERE namespace_id = ? AND id = ?",
            )
            .bind(content)
            .bind(description)
            .bind(namespace_id)
            .bind(config_id)
            .execute(&self.pool)
            .await?;
        }

        // 添加历史记录
        let config = self.get_config(namespace_id, config_id).await?.unwrap();
        self.append_history(&config).await?;

        Ok(())
    }

    pub async fn delete_config_and_sync(
        &self,
        namespace_id: &str,
        config_id: &str,
    ) -> anyhow::Result<()> {
        self.delete_config(namespace_id, config_id).await?;

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
        // 保存历史
        sqlx::query(
            "INSERT INTO config_history (namespace_id, id, content, description, ts) VALUES (?, ?, ?, ?, ?)",
        ).bind(&entry.namespace_id)
            .bind(&entry.id)
            .bind(&entry.content)
            .bind(&entry.description)
            .bind(&entry.ts)
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

    pub async fn recovery(&self, id_: i64) -> anyhow::Result<()> {
        let history: ConfigEntry = sqlx::query_as("SELECT * FROM config WHERE id_ = ?")
            .bind(id_)
            .fetch_one(&self.pool)
            .await?;
        self.upsert_config(
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
        self.network
            .post(format!("http://127.0.0.1:{}/write", self.port))
            .json(&request)
            .send()
            .await?;
        log::info!("sync config success");
        Ok(())
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
        let cm = ConfigManager::new(8000, "sqlite::memory:").await.unwrap();
        let config = cm.get_config("default", "test").await.unwrap();
        println!("config: {:?}", config);

        cm.upsert_config("default", "test", "name: 1", None)
            .await
            .unwrap();

        let config = cm.get_config("default", "test").await.unwrap();
        println!("config: {:?}", config);

        cm.upsert_config("default", "test", "name: 2", None)
            .await
            .unwrap();

        let config = cm.get_config("default", "test").await.unwrap();
        println!("config: {:?}", config);

        let history = cm.get_history("default", "test").await.unwrap();
        println!("history: {:?}", history);

        cm.recovery(1).await.unwrap();
        let config = cm.get_config("default", "test").await.unwrap();
        println!("config: {:?}", config);
        let history = cm.get_history("default", "test").await.unwrap();
        println!("history: {:?}", history);
    }
}
