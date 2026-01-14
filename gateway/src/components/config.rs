use crate::components::client::INNER_HTTP_CLIENT;
use aiway_protocol::gateway::Config;
use std::process::exit;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::RwLock;

pub struct ConfigFactory {
    config: Arc<RwLock<Config>>,
}

static CONFIG: OnceLock<ConfigFactory> = OnceLock::new();

impl ConfigFactory {
    pub async fn init() {
        if let Err(e) = Self::load().await {
            log::error!("{}", e);
            exit(1)
        }
    }

    async fn load() -> anyhow::Result<()> {
        let config = Self::fetch_config().await?;
        log::info!("loaded config: {:?}", config);

        CONFIG.get_or_init(|| Self {
            config: Arc::new(RwLock::new(config)),
        });

        Self::watch();

        Ok(())
    }

    async fn fetch_config() -> anyhow::Result<Config> {
        INNER_HTTP_CLIENT.fetch_config().await
    }

    const INTERVAL: Duration = Duration::from_secs(5);

    fn watch() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Self::INTERVAL);
            loop {
                interval.tick().await;

                let config = Self::fetch_config().await;

                let config = match config {
                    Ok(list) => list,
                    Err(e) => {
                        log::error!("{}", e);
                        continue;
                    }
                };

                let old_config = CONFIG.get().unwrap();
                if old_config.config.read().await.eq(&config) {
                    log::debug!("services not changed, wait next interval");
                    continue;
                }

                log::info!("loaded new config: {:?}", config);

                {
                    *old_config.config.write().await = config;
                }
            }
        });
    }
}
