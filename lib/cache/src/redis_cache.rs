use anyhow::anyhow;
use async_trait::async_trait;
use deadpool_redis::Runtime;
use deadpool_redis::redis::{AsyncTypedCommands, IntegerReplyOrNoOp};
use serde_json::Value;

/// 单节点的Redis缓存
pub struct RedisCache {
    pool: deadpool_redis::Pool,
}

impl RedisCache {
    pub fn new(url: String) -> anyhow::Result<Self> {
        let cfg = deadpool_redis::Config::from_url(url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        Ok(Self { pool })
    }
}

macro_rules! redis_set_ex {
    ($pool:expr, $key:expr, $value:expr, $ttl:expr) => {{
        let mut conn = $pool.get().await?;
        match $ttl {
            Some(ttl) => {
                conn.set_ex($key, serde_json::to_string($value)?, ttl)
                    .await?;
            }
            None => {
                conn.set($key, serde_json::to_string($value)?).await?;
            }
        }
        Ok(())
    }};
}

macro_rules! redis_get {
    ($pool:expr, $key:expr) => {{
        let mut conn = $pool.get().await?;
        let result: Option<String> = conn.get($key).await?;

        match result {
            Some(value) => {
                let parsed_value: Value = serde_json::from_str(&value)
                    .map_err(|e| anyhow!("Failed to parse JSON: {}", e))?;
                Ok(Some(parsed_value))
            }
            None => Ok(None),
        }
    }};
}

macro_rules! redis_del {
    ($pool:expr, $key:expr) => {{
        let mut conn = $pool.get().await?;
        conn.del($key).await?;
        Ok(())
    }};
}

macro_rules! redis_incr {
    ($pool:expr, $key:expr, $value:expr) => {{
        let mut conn = $pool.get().await?;
        let result = conn.incr($key, $value).await?;
        Ok(result as i64)
    }};
}
macro_rules! redis_ttl {
    ($pool:expr, $key:expr) => {{
        let mut conn = $pool.get().await?;
        let ttl_result = conn.ttl($key).await?;
        match ttl_result {
            IntegerReplyOrNoOp::IntegerReply(ttl) => Ok(ttl as i64),
            IntegerReplyOrNoOp::NotExists => Ok(-2),
            _ => Err(anyhow!("TTL operation returned unexpected result")),
        }
    }};
}
macro_rules! redis_expire {
    ($pool:expr, $key:expr, $ttl:expr) => {{
        let mut conn = $pool.get().await?;
        conn.expire($key, $ttl).await?;
        Ok(())
    }};
}

macro_rules! redis_exists {
    ($pool:expr, $key:expr) => {{
        let mut conn = $pool.get().await?;
        let exists: bool = conn
            .exists($key)
            .await
            .map_err(|e| anyhow!("Redis error: {}", e))?;
        Ok(exists)
    }};
}

macro_rules! redis_ratelimit {
    ($pool:expr, $key:expr, $limit:expr, $time_window:expr) => {{
        let mut conn = $pool.get().await?;

        // 检查键是否存在
        let exists: bool = conn
            .exists($key)
            .await
            .map_err(|e| anyhow!("Redis error: {}", e))?;

        // 增加计数器
        let count = conn
            .incr($key, 1)
            .await
            .map_err(|e| anyhow!("Redis error: {}", e))? as i32;

        // 如果键不存在，设置过期时间
        if !exists {
            conn.expire($key, $time_window as i64)
                .await
                .map_err(|e| anyhow!("Redis error: {}", e))?;
        }

        // 判断是否超过限流阈值
        Ok(count > $limit)
    }};
}

macro_rules! redis_lock {
    ($pool:expr, $key:expr, $ttl:expr) => {{
        let mut conn = $pool.get().await?;
        // 检查键是否存在
        if conn.exists($key).await? {
            Err(anyhow::anyhow!("Key {} is locked", $key))
        } else {
            // 设置键并附加过期时间
            conn.set_ex($key, "", $ttl).await?;
            Ok(())
        }
    }};
}

macro_rules! redis_unlock {
    ($pool:expr, $key:expr) => {{
        let mut conn = $pool.get().await?;
        conn.del($key).await?;
        Ok(())
    }};
}
#[async_trait]
impl crate::Cache for RedisCache {
    async fn set(&self, key: String, value: &Value, ttl: Option<u64>) -> anyhow::Result<()> {
        redis_set_ex!(self.pool, key, value, ttl)
    }

    async fn get(&self, key: &str) -> anyhow::Result<Option<Value>> {
        redis_get!(self.pool, key)
    }

    async fn remove(&self, key: &str) -> anyhow::Result<()> {
        redis_del!(self.pool, key)
    }

    async fn ttl(&self, key: &str) -> anyhow::Result<i64> {
        redis_ttl!(self.pool, key)
    }

    async fn exists(&self, key: &str) -> anyhow::Result<bool> {
        redis_exists!(self.pool, key)
    }

    async fn increment(&self, key: &str, value: i64) -> anyhow::Result<i64> {
        redis_incr!(self.pool, key, value)
    }

    async fn expire(&self, key: &str, ttl: i64) -> anyhow::Result<()> {
        redis_expire!(self.pool, key, ttl)
    }

    async fn ratelimit(&self, key: &str, limit: i32, time_window: i32) -> anyhow::Result<bool> {
        redis_ratelimit!(self.pool, key, limit, time_window)
    }

    async fn lock(&self, key: &str, ttl: u64) -> anyhow::Result<()> {
        redis_lock!(self.pool, key, ttl)
    }

    async fn unlock(&self, key: &str) -> anyhow::Result<()> {
        redis_unlock!(self.pool, key)
    }
}

/// 集群模式的Redis缓存
pub struct RedisClusterCache {
    pool: deadpool_redis::cluster::Pool,
}

impl RedisClusterCache {
    pub fn new(urls: Vec<String>) -> anyhow::Result<Self> {
        let cfg = deadpool_redis::cluster::Config::from_urls(urls);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        Ok(Self { pool })
    }
}
#[async_trait]
impl crate::Cache for RedisClusterCache {
    async fn set(&self, key: String, value: &Value, ttl: Option<u64>) -> anyhow::Result<()> {
        redis_set_ex!(self.pool, key, value, ttl)
    }

    async fn get(&self, key: &str) -> anyhow::Result<Option<Value>> {
        redis_get!(self.pool, key)
    }

    async fn remove(&self, key: &str) -> anyhow::Result<()> {
        redis_del!(self.pool, key)
    }

    async fn ttl(&self, key: &str) -> anyhow::Result<i64> {
        redis_ttl!(self.pool, key)
    }

    async fn exists(&self, key: &str) -> anyhow::Result<bool> {
        redis_exists!(self.pool, key)
    }

    async fn increment(&self, key: &str, value: i64) -> anyhow::Result<i64> {
        redis_incr!(self.pool, key, value)
    }

    async fn expire(&self, key: &str, ttl: i64) -> anyhow::Result<()> {
        redis_expire!(self.pool, key, ttl)
    }

    async fn ratelimit(&self, key: &str, limit: i32, time_window: i32) -> anyhow::Result<bool> {
        redis_ratelimit!(self.pool, key, limit, time_window as i64)
    }

    async fn lock(&self, key: &str, ttl: u64) -> anyhow::Result<()> {
        redis_lock!(self.pool, key, ttl)
    }

    async fn unlock(&self, key: &str) -> anyhow::Result<()> {
        redis_unlock!(self.pool, key)
    }
}
