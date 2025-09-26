use crate::Cache;
use crate::local_cache::LocalCache;
use serde_json::Value;
use std::future::pending;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use zbus::{connection, interface};

#[derive(Debug, Clone)]
pub struct ShareCacheServer {
    local_cache: Arc<LocalCache>,
}

/// zbus服务
#[interface(name = "aiway.share.cache")]
impl ShareCacheServer {
    async fn set(
        &self,
        key: String,
        value: String,
        ttl: (bool, u64),
    ) -> Result<(), zbus::fdo::Error> {
        self.local_cache
            .set(
                key,
                &Value::from_str(&value).unwrap(),
                if ttl.0 { Some(ttl.1) } else { None },
            )
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    async fn get(&self, key: &str) -> Result<(bool, String), zbus::fdo::Error> {
        match self.local_cache.get(key) {
            Some(value) => Ok((true, value.to_string())),
            None => Ok((false, "".to_string())),
        }
    }

    async fn remove(&self, key: &str) {
        self.local_cache.remove(key).unwrap();
    }

    async fn ttl(&self, key: &str) -> Result<i64, zbus::fdo::Error> {
        self.local_cache
            .ttl(key)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }
}

/// 启动zbus服务
///
/// **注意：该方法会阻塞执行，勿在主线程中直接调用，使用tokio异步执行**
pub async fn start_share_cache_server<P: AsRef<Path>>(db_path: P) -> anyhow::Result<()> {
    let local_cache = Arc::new(LocalCache::new(db_path)?);
    let server = ShareCacheServer { local_cache };
    let _conn = connection::Builder::session()?
        .name("aiway.share")?
        .serve_at("/aiway/share/cache", server)?
        .build()
        .await?;
    log::info!("share cache server started");
    pending::<()>().await;
    Ok(())
}
