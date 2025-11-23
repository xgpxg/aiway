use async_trait::async_trait;
use moka::sync::Cache;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// 缓存KEY
    pub k: String,
    /// 缓存值
    pub v: Value,
    /// 创建时间
    pub ct: u64,
    /// 过期时间, -1表示不过期
    pub ttl: i64,
}

#[derive(Debug)]
pub struct LocalCache {
    memory_cache: Cache<String, CacheEntry>,
    disk_db: sled::Db,
}

impl LocalCache {
    pub fn new<P: AsRef<Path>>(db_path: P) -> anyhow::Result<LocalCache> {
        let db = sled::open(db_path)?;
        let cache = Cache::builder()
            // 最大容量：10万
            // 超出容量的会从内存中移除
            // 如果移除时仍然没有过期，在get时会从磁盘加载，重新放入内存
            .max_capacity(100_000)
            .build();

        let persistent_cache = Self {
            memory_cache: cache,
            disk_db: db,
        };

        // 从磁盘加载
        persistent_cache.load_from_disk()?;

        Ok(persistent_cache)
    }

    fn get_cache_entry(&self, key: &str) -> Option<CacheEntry> {
        // 从内存缓存中获取
        if let Some(entry) = self.memory_cache.get(key) {
            // 已过期，同时删除内存缓存和磁盘中的
            if self.is_expired(&entry) {
                self.memory_cache.remove(key);
                let _ = self.disk_db.remove(key.as_bytes());
                return None;
            }
            return Some(entry);
        }

        // 如果内存中没有，从磁盘获取
        // 这种情况会出现在内存缓存已满，被移除了内存，但是缓存还没有过期
        // 如果过期，则从磁盘中删除
        if let Ok(Some(data)) = self.disk_db.get(key.as_bytes())
            && let Ok(entry) = serde_json::from_slice::<CacheEntry>(&data)
        {
            if !self.is_expired(&entry) {
                self.memory_cache.insert(key.to_string(), entry.clone());
                return Some(entry);
            } else {
                let _ = self.disk_db.remove(key.as_bytes());
            }
        }

        None
    }

    pub fn insert(&self, key: String, value: &Value, ttl: Option<u64>) -> anyhow::Result<()> {
        let entry = CacheEntry {
            k: key.clone(),
            v: value.clone(),
            ct: Self::current_time(),
            ttl: if let Some(ttl) = ttl { ttl as i64 } else { -1 },
        };

        // 保存到内存缓存
        self.memory_cache.insert(key.clone(), entry.clone());

        // 异步刷盘
        let db = self.disk_db.clone();
        tokio::spawn(async move {
            let serialized = serde_json::to_vec(&entry).unwrap();
            db.insert(key.as_bytes(), serialized).unwrap();
        });

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        match self.get_cache_entry(key) {
            Some(entry) => Some(entry.v),
            None => None,
        }
    }

    pub fn remove(&self, key: &str) -> anyhow::Result<()> {
        self.memory_cache.remove(key);
        let _ = self.disk_db.remove(key.as_bytes());
        Ok(())
    }

    pub fn ttl(&self, key: &str) -> anyhow::Result<i64> {
        match self.get_cache_entry(key) {
            Some(entry) => Ok(entry.ttl),
            None => Ok(-2),
        }
    }

    pub fn exists(&self, key: &str) -> anyhow::Result<bool> {
        Ok(self.get_cache_entry(key).is_some())
    }

    pub fn increment(&self, key: String, value: i64) -> anyhow::Result<i64> {
        // 获取当前值
        let mut entry = match self.get_cache_entry(&key) {
            Some(entry) => entry,
            None => CacheEntry {
                k: key.clone(),
                v: serde_json::to_value(0)?,
                ct: Self::current_time(),
                ttl: -1,
            },
        };

        // 检查值是否为数值类型
        let current_value = match entry.v.as_i64() {
            Some(val) => val,
            None => {
                return Err(anyhow::anyhow!("Value is not a valid integer"));
            }
        };

        let new_value = current_value + value;
        entry.v = serde_json::to_value(new_value)?;

        // 更新内存缓存
        self.memory_cache.insert(key.clone(), entry.clone());
        // 异步刷盘
        let db = self.disk_db.clone();
        tokio::spawn(async move {
            let serialized = serde_json::to_vec(&entry).unwrap();
            db.insert(key.as_bytes(), serialized).unwrap();
        });

        Ok(new_value)
    }

    pub fn expire(&self, key: String, ttl: i64) -> anyhow::Result<()> {
        if let Some(mut entry) = self.get_cache_entry(&key) {
            entry.ttl = ttl;
            self.memory_cache.insert(key.clone(), entry.clone());
            // 异步刷盘
            let db = self.disk_db.clone();
            tokio::spawn(async move {
                let serialized = serde_json::to_vec(&entry).unwrap();
                db.insert(key.as_bytes(), serialized).unwrap();
            });
        }
        Ok(())
    }

    fn current_time() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn is_expired(&self, entry: &CacheEntry) -> bool {
        if entry.ttl == -1 {
            return false;
        }
        (Self::current_time() - entry.ct) as i64 >= entry.ttl
    }

    fn load_from_disk(&self) -> anyhow::Result<()> {
        let now = Self::current_time();

        for result in self.disk_db.iter() {
            let (key, value) = result?;
            #[allow(clippy::collapsible_if)]
            if let Ok(key_str) = std::str::from_utf8(&key) {
                if let Ok(entry) = serde_json::from_slice::<CacheEntry>(&value) {
                    if self.is_expired(&entry) {
                        let _ = self.disk_db.remove(key);
                    } else {
                        self.memory_cache.insert(key_str.to_string(), entry);
                    }
                }
            }
        }

        log::trace!("cache: {:#?}", self.memory_cache);
        log::info!(
            "Loaded {} entries from disk, use {} seconds",
            self.memory_cache.iter().count(),
            Self::current_time() - now
        );
        Ok(())
    }

    /// 未过期的缓存写入到磁盘
    pub(crate) fn sync_to_disk(&self) {
        let db = self.disk_db.clone();
        for (key, entry) in self.memory_cache.iter() {
            if !self.is_expired(&entry) {
                let serialized = serde_json::to_vec(&entry).unwrap();
                db.insert(key.as_bytes(), serialized).unwrap();
                db.flush().unwrap();
            }
        }
    }

    pub fn ratelimit(&self, key: &str, limit: i32, time_window: i32) -> anyhow::Result<bool> {
        let exists = self.exists(key)?;
        let count = self.increment(key.to_string(), 1)?;
        if !exists {
            self.expire(key.to_string(), time_window as i64)?;
        }
        Ok(count > limit as i64)
    }
}

impl Drop for LocalCache {
    fn drop(&mut self) {
        log::info!("application shutdown, waiting sync memory cache to disk");
        self.sync_to_disk()
    }
}
#[async_trait]
impl crate::Cache for LocalCache {
    async fn set(&self, key: String, value: &Value, ttl: Option<u64>) -> anyhow::Result<()> {
        self.insert(key, value, ttl)
    }

    async fn get(&self, key: &str) -> anyhow::Result<Option<Value>> {
        Ok(self.get(key))
    }

    async fn remove(&self, key: &str) -> anyhow::Result<()> {
        self.remove(key)
    }

    async fn ttl(&self, key: &str) -> anyhow::Result<i64> {
        self.ttl(key)
    }

    async fn exists(&self, key: &str) -> anyhow::Result<bool> {
        self.exists(key)
    }

    async fn increment(&self, key: &str, value: i64) -> anyhow::Result<i64> {
        self.increment(key.to_string(), value)
    }

    async fn expire(&self, key: &str, ttl: i64) -> anyhow::Result<()> {
        self.expire(key.to_string(), ttl)
    }

    async fn ratelimit(&self, key: &str, limit: i32, time_window: i32) -> anyhow::Result<bool> {
        self.ratelimit(key, limit, time_window)
    }

    async fn lock(&self, _key: &str, _ttl: u64) -> anyhow::Result<()> {
        Ok(())
    }

    async fn unlock(&self, _key: &str) -> anyhow::Result<()> {
        Ok(())
    }
}
