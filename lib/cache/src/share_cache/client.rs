use async_trait::async_trait;
use serde_json::Value;
use zbus::proxy;

#[proxy(
    interface = "aiway.share.cache",
    default_service = "aiway.share",
    default_path = "/aiway/share/cache",
    gen_blocking = true
)]
trait ShareCacheClient {
    async fn set(&self, key: String, value: String, ttl: (bool, u64)) -> anyhow::Result<()>;

    async fn get(&self, key: &str) -> anyhow::Result<(bool, String)>;

    async fn remove(&self, key: &str) -> anyhow::Result<()>;

    async fn ttl(&self, key: &str) -> anyhow::Result<i64>;
}

#[derive(Debug, Clone)]
pub struct ShareCache {
    proxy: ShareCacheClientProxy<'static>,
}

impl ShareCache {
    pub async fn new() -> anyhow::Result<Self> {
        let connection = zbus::Connection::session().await?;
        let proxy = ShareCacheClientProxy::new(&connection).await?;
        Ok(Self { proxy })
    }
}

// impl Drop for ShareCache {
//     fn drop(&mut self) {
//         self.local_cache.sync_to_disk();
//     }
// }

#[async_trait]
impl crate::Cache for ShareCache {
    async fn set(&self, key: String, value: &Value, ttl: Option<u64>) -> anyhow::Result<()> {
        self.proxy
            .set(
                key,
                value.to_string(),
                match ttl {
                    Some(ttl) => (true, ttl),
                    None => (false, 0),
                },
            )
            .await
    }

    async fn get(&self, key: &str) -> anyhow::Result<Option<Value>> {
        match self.proxy.get(key).await {
            Ok(value) => match value.0 {
                true => Ok(Some(serde_json::from_str(&value.1)?)),
                false => Ok(None),
            },
            Err(err) => Err(anyhow::anyhow!(err)),
        }
    }

    async fn remove(&self, key: &str) -> anyhow::Result<()> {
        self.proxy.remove(key).await
    }

    async fn ttl(&self, key: &str) -> anyhow::Result<i64> {
        self.proxy.ttl(key).await
    }

    async fn exists(&self, _key: &str) -> anyhow::Result<bool> {
        unimplemented!()
    }

    async fn increment(&self, _key: &str, _value: i64) -> anyhow::Result<i64> {
        unimplemented!()
    }

    async fn expire(&self, _key: &str, _ttl: i64) -> anyhow::Result<()> {
        unimplemented!()
    }

    async fn ratelimit(&self, _key: &str, _limit: i32, _time_window: i32) -> anyhow::Result<bool> {
        unimplemented!()
    }

    async fn lock(&self, _key: &str, _ttl: u64) -> anyhow::Result<()> {
        unimplemented!()
    }

    async fn unlock(&self, _key: &str) -> anyhow::Result<()> {
        unimplemented!()
    }
}
