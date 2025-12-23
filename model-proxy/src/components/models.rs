use crate::components::client::INNER_HTTP_CLIENT;
use dashmap::DashMap;
use logging::log;
use protocol::model::Model;
use protocol::model::Provider;
use std::process::exit;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::RwLock;

pub struct ModelFactory {
    models: DashMap<String, Model>,
    hash: Arc<RwLock<String>>,
}

pub static MODEL_FACTORY: OnceLock<ModelFactory> = OnceLock::new();

impl ModelFactory {
    pub async fn init() {
        if let Err(e) = Self::load().await {
            log::error!("{}", e);
            exit(1)
        }
    }

    pub async fn load() -> anyhow::Result<()> {
        let models = Self::fetch_models().await?;

        log::info!("loaded {} models", models.len());

        let hash = md5::compute(serde_json::to_string(&models)?);
        let hash = format!("{:x}", hash);

        MODEL_FACTORY.get_or_init(|| Self {
            models: models
                .into_iter()
                .map(|model| (model.name.clone(), model))
                .collect::<_>(),
            hash: Arc::new(RwLock::new(hash)),
        });

        Self::watch();

        Ok(())
    }

    async fn fetch_models() -> anyhow::Result<Vec<Model>> {
        INNER_HTTP_CLIENT.fetch_models().await
    }

    const INTERVAL: Duration = Duration::from_secs(5);

    fn watch() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Self::INTERVAL);
            loop {
                interval.tick().await;
                let list = Self::fetch_models().await;

                let list = match list {
                    Ok(list) => list,
                    Err(e) => {
                        log::error!("fetch models error: {}", e);
                        continue;
                    }
                };

                let hash = md5::compute(serde_json::to_string(&list).unwrap());
                let hash = format!("{:x}", hash);

                let old = MODEL_FACTORY.get().unwrap();

                if hash == *old.hash.read().await {
                    log::debug!("models not changed, wait next interval");
                    continue;
                }

                log::info!("loaded {} models", list.len());

                {
                    list.into_iter().for_each(|model| {
                        old.models.insert(model.name.clone(), model);
                    });
                    *old.hash.write().await = hash;
                }
            }
        });
    }

    /// 获取模型的提供商
    pub fn get_provider(model_name: &str) -> Option<Provider> {
        let factory = MODEL_FACTORY.get().unwrap();
        let model = factory.models.get(model_name);
        match model {
            Some(model) => {
                let providers = &model.providers;
                let len = providers.len();
                if len > 0 {
                    let index = fastrand::usize(0..len);
                    return Some(providers[index].clone());
                }
                log::warn!("model {} has no providers", model_name);
                None
            }
            None => {
                log::warn!("model {} not found", model_name);
                None
            }
        }
    }
}
