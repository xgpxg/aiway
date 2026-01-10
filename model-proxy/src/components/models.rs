use crate::components::client::INNER_HTTP_CLIENT;
use crate::proxy::{ModelError, Proxy};
use dashmap::DashMap;
use logging::log;
use aiway_protocol::model::Provider;
use aiway_protocol::model::{LbStrategy, Model};
use std::collections::HashMap;
use std::process::exit;
use std::sync::OnceLock;
use std::time::Duration;

pub struct ModelFactory {
    /// 模型列表
    /// - key: 模型名称
    /// - value: 模型对象
    models: DashMap<String, Model>,
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
        MODEL_FACTORY.get_or_init(|| Self {
            models: models
                .into_iter()
                .map(|model| (model.name.clone(), model))
                .collect::<_>(),
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

                let old = MODEL_FACTORY.get().unwrap();

                let new_models = list
                    .into_iter()
                    .map(|m| (m.name.clone(), m))
                    .collect::<HashMap<String, Model>>();

                // 移除不存在的
                old.models.retain(|_, item| {
                    if !new_models.contains_key(&item.name) {
                        log::info!("removed model: {}", item.name);
                        Proxy::remove_clients(&item.name);
                        return false;
                    }
                    true
                });

                // 处理新增和变更的
                new_models.into_iter().for_each(|(name, new_model)| {
                    let need_update = match old.models.get(&name) {
                        Some(old_model) => old_model.ne(&new_model),
                        None => true,
                    };

                    if need_update {
                        if old.models.get(&name).is_some() {
                            log::info!("changed model: {}", name);
                        } else {
                            log::info!("new model enabled: {}", name);
                        }
                        old.models.insert(name.clone(), new_model);
                        Proxy::remove_clients(&name);
                    }
                });
            }
        });
    }

    /// 按负载策略获取模型的提供商
    pub fn get_provider(model_name: &str) -> Result<Provider, ModelError> {
        let factory = MODEL_FACTORY.get().unwrap();
        let model = factory.models.get_mut(model_name);
        match model {
            Some(mut model) => {
                let model = model.value_mut();
                log::debug!(
                    "get provider for model: {}, model detail: {:?}",
                    model_name,
                    model
                );
                let providers = &model.providers;
                let len = providers.len();
                if len == 0 {
                    return Err(ModelError::NoAvailableProvider);
                }
                if len == 1 {
                    return Ok(providers[0].clone());
                }
                match &model.lb {
                    // 随机
                    LbStrategy::Random => {
                        let index = fastrand::usize(0..len);
                        Ok(providers[index].clone())
                    }
                    // 轮询
                    LbStrategy::RoundRobin => {
                        let index = model.round_robin_index % (len as u64);
                        model.round_robin_index = index + 1;
                        Ok(providers[index as usize].clone())
                    }
                    // 权重随机
                    LbStrategy::WeightedRandom => {
                        let mut random_weight = fastrand::u32(0..model.total_weight);
                        for provider in providers {
                            if random_weight < provider.weight {
                                return Ok(provider.clone());
                            }
                            random_weight -= provider.weight;
                        }
                        // 理论上不会到达这里，但作为安全fallback
                        Ok(providers[0].clone())
                    }
                }
            }
            None => Err(ModelError::UnsupportedModel(model_name.to_string())),
        }
    }
}
