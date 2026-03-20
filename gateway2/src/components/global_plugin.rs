use crate::components::client::INNER_HTTP_CLIENT;
use aiway_protocol::gateway::{ConfiguredPlugin, GlobalPlugin};
use anyhow::Context;
use std::process::exit;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Duration;

pub struct GlobalPluginFactory {
    pub config: Arc<RwLock<GlobalPlugin>>,
    hash: Arc<RwLock<String>>,
}

pub static GLOBAL_PLUGIN: OnceLock<GlobalPluginFactory> = OnceLock::new();

impl GlobalPluginFactory {
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

        GLOBAL_PLUGIN.get_or_init(|| Self {
            config: Arc::new(RwLock::new(config)),
            hash: Arc::new(RwLock::new(hash)),
        });

        Self::watch();

        Ok(())
    }

    async fn fetch_config() -> anyhow::Result<GlobalPlugin> {
        INNER_HTTP_CLIENT.fetch_global_plugins().await
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

                let old_config = GLOBAL_PLUGIN.get().unwrap();

                if *old_config.hash.read().unwrap() == hash {
                    log::debug!("gateway global filters not changed, wait next interval");
                    continue;
                }

                log::info!("loaded global filters config: {:?}", config);

                {
                    *old_config.config.write().unwrap() = config;
                    *old_config.hash.write().unwrap() = hash;
                }
            }
        });
    }

    pub fn get_plugins() -> Vec<ConfiguredPlugin> {
        GLOBAL_PLUGIN
            .get()
            .unwrap()
            .config
            .read()
            .unwrap()
            .plugins
            .clone()
    }
}
