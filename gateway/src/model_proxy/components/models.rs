use crate::model_proxy::components::client::INNER_HTTP_CLIENT;
use crate::model_proxy::proxy::{ModelError, Proxy};
use aiway_protocol::model::Provider;
use aiway_protocol::model::{LbStrategy, Model};
use dashmap::DashMap;
use logging::log;
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

    /// 按负载策略获取模型的候选提供商列表，用于故障自动切换
    pub fn get_providers(model_name: &str) -> Result<Vec<Provider>, ModelError> {
        let factory = MODEL_FACTORY.get().unwrap();
        let mut model = match factory.models.get_mut(model_name) {
            Some(m) => m,
            None => return Err(ModelError::UnsupportedModel(model_name.to_string())),
        };
        let model = model.value_mut();
        let providers = &model.providers;
        if providers.is_empty() {
            return Err(ModelError::NoAvailableProvider);
        }
        if providers.len() == 1 {
            return Ok(providers.clone());
        }
        let sorted = match &model.lb {
            // 随机：重新打乱顺序
            LbStrategy::Random => {
                let mut list = providers.clone();
                fastrand::shuffle(&mut list);
                list
            }
            // 轮询：重新排序
            LbStrategy::RoundRobin => {
                let len = providers.len();
                let start = (model.round_robin_index % len as u64) as usize;
                model.round_robin_index += 1;
                let mut list = providers[start..].to_vec();
                list.extend_from_slice(&providers[..start]);
                list
            }
            // 权重随机（指数分布排序，权重越大越靠前）
            LbStrategy::WeightedRandom => {
                let mut list: Vec<_> = providers
                    .iter()
                    .map(|p| {
                        let u = fastrand::f64().max(f64::MIN_POSITIVE);
                        (-(u.ln() / p.weight as f64), p)
                    })
                    .collect();
                list.sort_by(|a, b| a.0.total_cmp(&b.0));
                list.into_iter().map(|(_, p)| p.clone()).collect()
            }
        };
        Ok(sorted)
    }
    pub fn get_special_provider(model_name: &str, provider_name: &str) -> Option<Provider> {
        let factory = MODEL_FACTORY.get().unwrap();
        let model = factory.models.get(model_name);
        match model {
            Some(model) => {
                let model = model.value();
                log::debug!(
                    "get provider for model: {}, model detail: {:?}",
                    model_name,
                    model
                );
                let providers = &model.providers;
                for provider in providers {
                    if provider.name == provider_name {
                        return Some(provider.clone());
                    }
                }
                None
            }
            None => None,
        }
    }
}
