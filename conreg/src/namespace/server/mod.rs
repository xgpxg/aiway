pub mod api;

use crate::Args;
use crate::config::server::{ConfigApp, ConfigManager};
use crate::raft::RaftRequest;
use chrono::{DateTime, Local};
use logging::log;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use std::process::exit;
use std::time::Duration;

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Namespace {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub ts: DateTime<Local>,
}

#[derive(Debug)]
pub struct NamespaceManager {
    /// 本地sqlite数据库，用于维护配置内容存储。
    /// 通过raft保证一致性
    pool: SqlitePool,
    /// Http客户端，主要用于同步log到集群
    http_client: reqwest::Client,
    /// 启动参数
    args: Args,
}

impl NamespaceManager {
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

        Ok(Self {
            pool,
            http_client: network,
            args: args.clone(),
        })
    }
    /// 初始化数据库
    async fn init(pool: &SqlitePool) -> anyhow::Result<()> {
        let sql = include_str!("../../db/init.sql");
        sqlx::query(sql).execute(pool).await?;
        Ok(())
    }

    pub async fn get_namespace(&self, id: &str) -> anyhow::Result<Option<Namespace>> {
        let namespace = sqlx::query_as("select * from namespace where name = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(namespace)
    }

    pub async fn upsert_namespace_and_sync(
        &self,
        id: &str,
        name: &str,
        description: Option<String>,
    ) -> anyhow::Result<()> {
        let namespace = Namespace {
            id: id.to_string(),
            name: name.to_string(),
            description: description.clone(),
            ts: Local::now(),
        };
        // 同步数据
        self.sync(RaftRequest::UpsertNamespace { namespace })
            .await?;
        Ok(())
    }

    pub async fn upsert_namespace(&self, namespace: Namespace) -> anyhow::Result<()> {
        let old = self.get_namespace(&namespace.id).await?;
        match old {
            None => {
                self.insert_namespace(&namespace).await?;
            }
            Some(_) => {
                self.update_namespace(&namespace).await?;
            }
        }
        Ok(())
    }
    async fn insert_namespace(&self, namespace: &Namespace) -> anyhow::Result<()> {
        sqlx::query("insert into namespace (id, name, description, ts) values (?, ?, ?, ?)")
            .bind(&namespace.id)
            .bind(&namespace.name)
            .bind(&namespace.description)
            .bind(namespace.ts)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_namespace(&self, namespace: &Namespace) -> anyhow::Result<()> {
        sqlx::query("update namespace set name = ?, description = ?, ts = ? where id = ?")
            .bind(&namespace.name)
            .bind(&namespace.description)
            .bind(&namespace.id)
            .bind(namespace.ts)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_namespace_and_sync(&self, id: &str) -> anyhow::Result<()> {
        self.sync(RaftRequest::DeleteNamespace { id: id.to_string() })
            .await?;
        Ok(())
    }

    pub async fn delete_namespace(&self, id: &str) -> anyhow::Result<()> {
        // 删除配置
        sqlx::query("delete from config where namespace_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        sqlx::query("delete from namespace where id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

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
}

#[derive(Debug)]
pub struct NamespaceApp {
    pub manager: NamespaceManager,
}
