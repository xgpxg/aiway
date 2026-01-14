//! # 缓存
//! 兼容本地缓存和Redis缓存。默认使用本地缓存。
//!
use crate::local_cache::LocalCache;
#[cfg(feature = "redis-cache")]
use crate::redis_cache::{RedisCache, RedisClusterCache};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::sync::OnceLock;

mod local_cache;

pub mod caches;
#[cfg(feature = "redis-cache")]
mod redis_cache;
#[cfg(feature = "share-cache")]
mod share_cache;
#[cfg(feature = "share-cache")]
pub use share_cache::start_share_cache_server;

#[allow(unused)]
#[async_trait]
pub trait Cache: Send + Sync {
    /// 设置缓存
    async fn set(&self, key: String, value: &Value, ttl: Option<u64>) -> anyhow::Result<()>;
    /// 获取缓存
    async fn get(&self, key: &str) -> anyhow::Result<Option<Value>>;
    /// 删除缓存
    async fn remove(&self, key: &str) -> anyhow::Result<()>;
    /// 获取缓存的剩余时间
    /// 特殊值：
    /// - -1：永不过期
    /// - -2：key不存在
    async fn ttl(&self, key: &str) -> anyhow::Result<i64>;
    /// 判断缓存是否存在
    async fn exists(&self, key: &str) -> anyhow::Result<bool>;
    /// 自增
    async fn increment(&self, key: &str, value: i64) -> anyhow::Result<i64>;
    /// 设置缓存的过期时间
    async fn expire(&self, key: &str, ttl: i64) -> anyhow::Result<()>;
    /// 限流
    async fn ratelimit(&self, key: &str, limit: i32, time_window: i32) -> anyhow::Result<bool>;
    /// 锁
    /// 简单实现的排他锁，主要用于防止定时任重复执行
    /// 除了定时任务外，尽量不要使用
    /// 锁的超时时间建议不要设置过长，不要超过30秒
    /// 对于单节点模式，该方法直接返回Ok
    async fn lock(&self, key: &str, ttl: u64) -> anyhow::Result<()>;
    /// 解锁
    async fn unlock(&self, key: &str) -> anyhow::Result<()>;
}

static CACHE: OnceLock<Box<dyn Cache>> = OnceLock::new();

pub fn init_local_cache<P: AsRef<Path>>(dir: P) -> anyhow::Result<()> {
    log::info!("init local cache");
    CACHE
        .set(Box::new(LocalCache::new(dir)?))
        .map_err(|_| anyhow::anyhow!("cache already initialized"))?;
    Ok(())
}

#[cfg(feature = "redis-cache")]
pub fn init_redis_cache<N: AsRef<str>>(nodes: Vec<N>) -> anyhow::Result<()> {
    if nodes.is_empty() {
        return Err(anyhow::anyhow!("redis nodes is empty"));
    }
    if nodes.len() == 1 {
        return init_single_redis_cache(nodes[0].as_ref());
    }
    init_cluster_redis_cache(
        nodes
            .into_iter()
            .map(|s| s.as_ref().to_string())
            .collect::<Vec<_>>(),
    )
}

#[cfg(feature = "redis-cache")]
fn init_single_redis_cache(url: &str) -> anyhow::Result<()> {
    log::info!("init redis cache");
    CACHE
        .set(Box::new(RedisCache::new(url.to_string())?))
        .map_err(|_| anyhow::anyhow!("cache already initialized"))?;
    Ok(())
}

#[cfg(feature = "redis-cache")]
fn init_cluster_redis_cache(nodes: Vec<String>) -> anyhow::Result<()> {
    log::info!("init redis cache");
    CACHE
        .set(Box::new(RedisClusterCache::new(nodes)?))
        .map_err(|_| anyhow::anyhow!("cache already initialized"))?;
    Ok(())
}

#[cfg(feature = "share-cache")]
pub async fn init_share_cache() -> anyhow::Result<()> {
    log::info!("init share cache");
    CACHE
        .set(Box::new(share_cache::ShareCache::new().await?))
        .map_err(|_| anyhow::anyhow!("cache already initialized"))?;
    Ok(())
}

pub async fn set<T: Serialize>(key: String, value: &T, ttl: Option<u64>) -> anyhow::Result<()> {
    let json_value = serde_json::to_value(value)?;
    if let Some(cache) = CACHE.get() {
        cache.set(key, &json_value, ttl).await
    } else {
        Err(anyhow::anyhow!("Cache not initialized"))
    }
}

pub async fn get<T: for<'de> Deserialize<'de>>(key: &str) -> anyhow::Result<Option<T>> {
    if let Some(cache) = CACHE.get() {
        match cache.get(key).await? {
            Some(value) => {
                let deserialized: T = serde_json::from_value(value)?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    } else {
        Err(anyhow::anyhow!("Cache not initialized"))
    }
}

pub async fn remove(key: &str) -> anyhow::Result<()> {
    if let Some(cache) = CACHE.get() {
        cache.remove(key).await
    } else {
        Err(anyhow::anyhow!("Cache not initialized"))
    }
}

pub async fn exists(key: &str) -> anyhow::Result<bool> {
    if let Some(cache) = CACHE.get() {
        cache.exists(key).await
    } else {
        Err(anyhow::anyhow!("Cache not initialized"))
    }
}

#[allow(unused)]
pub async fn ttl(key: &str) -> anyhow::Result<i64> {
    if let Some(cache) = CACHE.get() {
        cache.ttl(key).await
    } else {
        Err(anyhow::anyhow!("Cache not initialized"))
    }
}

pub async fn increment(key: &str, value: i64) -> anyhow::Result<i64> {
    if let Some(cache) = CACHE.get() {
        cache.increment(key, value).await
    } else {
        Err(anyhow::anyhow!("Cache not initialized"))
    }
}

pub async fn ratelimit(key: &str, limit: i32, time_window: i32) -> anyhow::Result<bool> {
    if let Some(cache) = CACHE.get() {
        cache.ratelimit(key, limit, time_window).await
    } else {
        Err(anyhow::anyhow!("Cache not initialized"))
    }
}

pub async fn lock(key: &str, ttl: u64) -> anyhow::Result<()> {
    if let Some(cache) = CACHE.get() {
        cache.lock(key, ttl).await
    } else {
        Err(anyhow::anyhow!("Cache not initialized"))
    }
}

pub async fn unlock(key: &str) -> anyhow::Result<()> {
    if let Some(cache) = CACHE.get() {
        cache.unlock(key).await
    } else {
        Err(anyhow::anyhow!("Cache not initialized"))
    }
}
