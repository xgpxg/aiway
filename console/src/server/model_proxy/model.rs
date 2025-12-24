use crate::server::db::Pool;
use crate::server::db::models::model::{Model, ModelStatus};
use crate::server::db::models::model_provider::{ModelProvider, ModelProviderStatus};
use rbs::value;
use std::collections::HashMap;

pub(crate) async fn models() -> anyhow::Result<Vec<protocol::model::Model>> {
    let tx = Pool::get()?;
    let models = Model::select_by_map(
        tx,
        value! {
            "status": ModelStatus::Ok
        },
    )
    .await?;
    if models.is_empty() {
        return Ok(vec![]);
    }

    let providers = ModelProvider::select_by_map(
        tx,
        value! {
            "status": ModelProviderStatus::Ok
        },
    )
    .await?;

    let providers_map = providers
        .into_iter()
        .fold(HashMap::new(), |mut map, provider| {
            map.entry(provider.model_id)
                .or_insert(vec![])
                .push(provider);
            map
        });
    let models = models
        .into_iter()
        .map(|model| {
            let providers = providers_map.get(&model.id);
            let providers = providers
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|provider| protocol::model::Provider {
                    name: provider.name.unwrap(),
                    api_url: provider.api_url.unwrap(),
                    api_key: provider.api_key,
                    weight: 1,
                    target_model_name: provider.target_model_name,
                })
                .collect::<Vec<_>>();
            let total_weight = providers.iter().map(|p| p.weight).sum::<u32>();
            protocol::model::Model {
                name: model.name.unwrap(),
                providers,
                lb: model.lb_strategy.unwrap(),
                round_robin_index: 0,
                total_weight,
            }
        })
        .collect();

    println!("models: {:?}", models);
    Ok(models)
}
