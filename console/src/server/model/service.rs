use crate::server::auth::UserPrincipal;
use crate::server::db::models::model::{LbStrategy, Model, ModelBuilder, ModelStatus};
use crate::server::db::models::model_provider::{
    ModelProvider, ModelProviderBuilder, ModelProviderStatus,
};
use crate::server::db::{Pool, tools};
use crate::server::model::request::{
    ModelAddReq, ModelLisReq, ModelUpdateReq, ProviderAddReq, ProviderUpdateReq,
};
use crate::server::model::response::ModelListRes;
use common::id;
use protocol::common::req::IdReq;
use rbs::value;
use std::collections::HashMap;

pub(crate) async fn list(_req: ModelLisReq) -> anyhow::Result<Vec<ModelListRes>> {
    let tx = Pool::get()?;
    let models = Model::select_all(tx).await?;
    if models.is_empty() {
        return Ok(vec![]);
    }

    let providers = ModelProvider::select_all(tx).await?;

    let providers_map = providers
        .into_iter()
        .fold(HashMap::new(), |mut map, provider| {
            map.entry(provider.model_id)
                .or_insert(vec![])
                .push(provider);
            map
        });

    Ok(models
        .into_iter()
        .map(|model| {
            let providers = providers_map.get(&model.id);
            ModelListRes {
                inner: model,
                providers: providers.cloned().unwrap_or_default(),
            }
        })
        .collect())
}

pub(crate) async fn add(req: ModelAddReq, user: UserPrincipal) -> anyhow::Result<()> {
    Model::insert(
        Pool::get()?,
        &ModelBuilder::default()
            .id(Some(id::next()))
            .name(Some(req.name))
            .status(Some(ModelStatus::Disable))
            .lb_strategy(Some(LbStrategy::Random))
            .create_time(Some(tools::now()))
            .create_user_id(Some(user.id))
            .build()?,
    )
    .await?;
    Ok(())
}

pub(crate) async fn update(req: ModelUpdateReq, user: UserPrincipal) -> anyhow::Result<()> {
    Model::update_by_map(
        Pool::get()?,
        &ModelBuilder::default()
            .name(req.name)
            .status(req.status)
            .lb_strategy(req.lb_strategy)
            .update_time(Some(tools::now()))
            .update_user_id(Some(user.id))
            .build()?,
        value! {
            "id": req.id
        },
    )
    .await?;
    Ok(())
}

pub(crate) async fn delete(req: IdReq) -> anyhow::Result<()> {
    Model::delete_by_map(
        Pool::get()?,
        value! {
            "id": req.id
        },
    )
    .await?;
    Ok(())
}

pub(crate) async fn add_provider(req: ProviderAddReq, user: UserPrincipal) -> anyhow::Result<()> {
    ModelProvider::insert(
        Pool::get()?,
        &ModelProviderBuilder::default()
            .id(Some(id::next()))
            .model_id(Some(req.model_id))
            .name(Some(req.name))
            .api_url(Some(req.api_url))
            .api_key(req.api_key)
            .status(Some(ModelProviderStatus::Disable))
            .create_time(Some(tools::now()))
            .create_user_id(Some(user.id))
            .build()?,
    )
    .await?;
    Ok(())
}

pub(crate) async fn update_provider(
    req: ProviderUpdateReq,
    user: UserPrincipal,
) -> anyhow::Result<()> {
    ModelProvider::update_by_map(
        Pool::get()?,
        &ModelProviderBuilder::default()
            .name(req.name)
            .api_url(req.api_url)
            .api_key(req.api_key)
            .status(req.status)
            .update_time(Some(tools::now()))
            .update_user_id(Some(user.id))
            .build()?,
        value! {
            "id": req.id
        },
    )
    .await?;
    Ok(())
}

pub(crate) async fn delete_provider(req: IdReq) -> anyhow::Result<()> {
    ModelProvider::delete_by_map(
        Pool::get()?,
        value! {
            "id": req.id
        },
    )
    .await?;
    Ok(())
}
