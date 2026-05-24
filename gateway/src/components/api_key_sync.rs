use crate::components::client::INNER_HTTP_CLIENT;
use crate::components::display_time_with_timestamp_millis;
use aiway_protocol::gateway::Action;
use cache::caches::CacheKey;
use serde_json::Value;
use std::process::exit;
use std::time::Duration;

pub struct ApiKeySyncer {}

impl ApiKeySyncer {
    pub async fn init() {
        if let Err(e) = Self::load().await {
            log::error!("{}", e);
            exit(1)
        }

        Self::watch();
    }

    pub async fn load() -> anyhow::Result<()> {
        let last_pull_time = cache::get::<i64>(&CacheKey::LastPullApiKeyTime.to_string()).await?;
        if let Some(last_pull_time) = last_pull_time {
            log::debug!(
                "last pull api key time: {:?}",
                display_time_with_timestamp_millis(last_pull_time)
            );
        } else {
            log::info!("no last pull api key time in cache, Pull full keys");
        }
        let list = INNER_HTTP_CLIENT.pull_api_key(last_pull_time).await?;

        let len = list.len();

        for ak in list {
            let key = CacheKey::ApiKey(ak.secret).to_string();
            let expire = ak
                .exp_time
                .map(|exp_time| exp_time - chrono::Utc::now().timestamp());
            let expire = expire.map(|expire| expire.max(0) as u64);
            match ak.action {
                Action::Create | Action::Update => cache::set(key, &Value::Null, expire).await?,
                Action::Delete => cache::remove(&key).await?,
            };
        }

        cache::set(
            CacheKey::LastPullApiKeyTime.to_string(),
            &chrono::Local::now().timestamp_millis(),
            None,
        )
        .await?;

        if len > 0 {
            log::info!("pull {} changed api keys", len);
        }
        Ok(())
    }

    const INTERVAL: Duration = Duration::from_secs(5);

    fn watch() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Self::INTERVAL);
            loop {
                interval.tick().await;

                if let Err(e) = Self::load().await {
                    log::error!("sync api key fail: {}", e);
                }
            }
        });
    }
}
