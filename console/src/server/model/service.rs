use crate::server::auth::UserPrincipal;
use crate::server::db::models::model::{Model, ModelBuilder, ModelStatus};
use crate::server::db::models::model_provider::{
    ModelProvider, ModelProviderBuilder, ModelProviderStatus,
};
use crate::server::db::{Pool, tools};
use crate::server::model::request::{
    ModelAddReq, ModelLisReq, ModelUpdateReq, ProviderAddReq, ProviderUpdateReq,
};
use crate::server::model::response::ModelListRes;
use anyhow::bail;
use common::id;
use busi::req::IdReq;
use aiway_protocol::model::LbStrategy;
use rbs::value;
use std::collections::HashMap;
use crate::update_nullable_fields;

pub(crate) async fn list(_req: ModelLisReq) -> anyhow::Result<Vec<ModelListRes>> {
    let tx = Pool::get()?;
    let mut models = Model::select_all(tx).await?;
    if models.is_empty() {
        return Ok(vec![]);
    }

    models.sort_by(|a, b| b.id.cmp(&a.id));

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
    if check_model_exists(&req.name, None).await? {
        bail!(format!("模型 {} 已存在", req.name));
    }
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

async fn check_model_exists(model_name: &str, exclude_id: Option<i64>) -> anyhow::Result<bool> {
    let mut list = Model::select_by_map(
        Pool::get()?,
        value! {
            "name": model_name,
        },
    )
    .await?;

    list.retain(|item| item.id != exclude_id);

    Ok(!list.is_empty())
}

pub(crate) async fn update(req: ModelUpdateReq, user: UserPrincipal) -> anyhow::Result<()> {
    if let Some(ref name) = req.name
        && check_model_exists(name, Some(req.id)).await?
    {
        bail!(format!("模型 {} 已存在", name));
    }
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
    if check_provider_exists(req.model_id, &req.name, None).await? {
        bail!(format!("提供商 {} 已存在", req.name));
    }
    ModelProvider::insert(
        Pool::get()?,
        &ModelProviderBuilder::default()
            .id(Some(id::next()))
            .model_id(Some(req.model_id))
            .name(Some(req.name))
            .api_url(Some(req.api_url))
            .api_key(req.api_key)
            .status(Some(ModelProviderStatus::Disable))
            .weight(req.weight)
            .target_model_name(req.target_model_name)
            .request_converter(req.request_converter)
            .response_converter(req.response_converter)
            .create_time(Some(tools::now()))
            .create_user_id(Some(user.id))
            .build()?,
    )
    .await?;
    Ok(())
}
async fn check_provider_exists(
    model_id: i64,
    provider_name: &str,
    exclude_id: Option<i64>,
) -> anyhow::Result<bool> {
    let mut list = ModelProvider::select_by_map(
        Pool::get()?,
        value! {
            "model_id": model_id,
            "name": provider_name,
        },
    )
    .await?;

    list.retain(|item| item.id != exclude_id);

    Ok(!list.is_empty())
}

pub(crate) async fn update_provider(
    req: ProviderUpdateReq,
    user: UserPrincipal,
) -> anyhow::Result<()> {
    let tx = Pool::get()?;
    let old = ModelProvider::select_by_map(
        tx,
        value! {
            "id": req.id
        },
    )
    .await?
    .pop()
    .ok_or(anyhow::anyhow!("提供商不存在"))?;
    if let Some(ref name) = req.name
        && check_provider_exists(old.model_id.unwrap(), name, Some(req.id)).await?
    {
        bail!(format!("提供商 {} 已存在", name));
    }
    ModelProvider::update_by_map(
        tx,
        &ModelProviderBuilder::default()
            .name(req.name)
            .api_url(req.api_url)
            .api_key(req.api_key)
            .status(req.status)
            .weight(req.weight)
            .target_model_name(req.target_model_name)
            .request_converter(req.request_converter.clone())
            .response_converter(req.response_converter.clone())
            .update_time(Some(tools::now()))
            .update_user_id(Some(user.id))
            .build()?,
        value! {
            "id": req.id
        },
    )
    .await?;

    update_nullable_fields!(
        tx,
        "model_provider",
        req.id,
        request_converter = req.request_converter,
        response_converter = req.response_converter
    );

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
