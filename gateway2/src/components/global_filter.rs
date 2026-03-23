use crate::components::client::INNER_HTTP_CLIENT;
use anyhow::Context;
use aiway_protocol::gateway::GlobalFilter;
use std::process::exit;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::RwLock;

pub struct GlobalFilterConfig {
    pub config: Arc<RwLock<GlobalFilter>>,
    hash: Arc<RwLock<String>>,
}

pub static GLOBAL_FILTER: OnceLock<GlobalFilterConfig> = OnceLock::new();

impl GlobalFilterConfig {
    pub async fn init() {
        if let Err(e) = Self::load().await {
            log::error!("{}", e);
            exit(1)
        }
    }

    pub async fn load() -> anyhow::Result<()> {
        let config = Self::fetch_config().await?;
        log::info!("loaded gateway global filters: {:?}", config);

        let hash = md5::compute(serde_json::to_string(&config)?);
        let hash = format!("{:x}", hash);

        GLOBAL_FILTER.get_or_init(|| Self {
            config: Arc::new(RwLock::new(config)),
            hash: Arc::new(RwLock::new(hash)),
        });

        Self::watch();

        Ok(())
    }

    async fn fetch_config() -> anyhow::Result<GlobalFilter> {
        INNER_HTTP_CLIENT.fetch_global_filter().await
    }

    const INTERVAL: Duration = Duration::from_secs(5);

    fn watch() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Self::INTERVAL);
            loop {
                interval.tick().await;
                let config = match Self::fetch_config().await {
                    Ok(config) => config,
                    Err(e) => {
                        log::error!("{}", e);
                        continue;
                    }
                };

                let hash = md5::compute(
                    serde_json::to_string(&config)
                        .context("serialize config")
                        .unwrap(),
                );
                let hash = format!("{:x}", hash);

                let old_config = GLOBAL_FILTER.get().unwrap();

                if *old_config.hash.read().await == hash {
                    log::debug!("gateway global filters not changed, wait next interval");
                    continue;
                }

                log::info!("loaded global filters config: {:?}", config);

                {
                    *old_config.config.write().await = config;
                    *old_config.hash.write().await = hash;
                }
            }
        });
    }
}
