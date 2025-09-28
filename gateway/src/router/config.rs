//! # 网关配置
//! 负责从控制台加载网关全局配置

use crate::router::client::INNER_HTTP_CLIENT;
use anyhow::Context;
use protocol::gateway::Configuration;
use std::process::exit;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::RwLock;

pub struct ConfigFactory {
    pub config: Arc<RwLock<Configuration>>,
    hash: Arc<RwLock<String>>,
}

pub static GATEWAY_CONFIG: OnceLock<ConfigFactory> = OnceLock::new();

impl ConfigFactory {
    pub async fn init() {
        if let Err(e) = Self::load().await {
            log::error!("{}", e);
            exit(1)
        }
    }

    pub async fn load() -> anyhow::Result<()> {
        let config = Self::fetch_configuration().await?;
        log::info!("loaded gateway config: {:?}", config);

        let hash = md5::compute(serde_json::to_string(&config)?);
        let hash = format!("{:x}", hash);

        GATEWAY_CONFIG.get_or_init(|| Self {
            config: Arc::new(RwLock::new(config)),
            hash: Arc::new(RwLock::new(hash)),
        });

        Self::watch();

        Ok(())
    }

    async fn fetch_configuration() -> anyhow::Result<Configuration> {
        INNER_HTTP_CLIENT.fetch_configuration().await
    }

    const INTERVAL: Duration = Duration::from_secs(5);

    fn watch() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Self::INTERVAL);
            loop {
                interval.tick().await;
                let config = match Self::fetch_configuration().await {
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

                let old_config = GATEWAY_CONFIG.get().unwrap();

                if *old_config.hash.read().await == hash {
                    log::debug!("gateway config not changed, wait next interval");
                    continue;
                }

                log::info!("loaded gateway config: {:?}", config);

                {
                    *old_config.config.write().await = config;
                    *old_config.hash.write().await = hash;
                }
            }
        });
    }
}
