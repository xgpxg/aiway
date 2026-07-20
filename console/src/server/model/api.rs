use crate::server::auth::UserPrincipal;
use crate::server::model::request::{
    ModelAddReq, ModelLisReq, ModelUpdateReq, ProviderAddReq, ProviderUpdateReq, RankReq, TrendReq,
};
use crate::server::model::response::{ModelListRes, ModelRankItem, TrendItem, UsageOverview};
use crate::server::model::{service, usage};
use busi::req::IdReq;
use busi::res::Res;
use rocket::serde::json::Json;
use rocket::{post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        list,
        add,
        update,
        delete,
        add_provider,
        update_provider,
        delete_provider,
        usage_overview,
        usage_trend,
        usage_model_rank,
    ]
}

/// 模型列表
#[post("/list", data = "<req>")]
pub async fn list(req: Json<ModelLisReq>, _user: UserPrincipal) -> Res<Vec<ModelListRes>> {
    match service::list(req.into_inner()).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 添加模型
#[post("/add", data = "<req>")]
pub async fn add(req: Json<ModelAddReq>, user: UserPrincipal) -> Res<()> {
    match service::add(req.into_inner(), user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 修改模型
#[post("/update", data = "<req>")]
pub async fn update(req: Json<ModelUpdateReq>, user: UserPrincipal) -> Res<()> {
    match service::update(req.into_inner(), user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/delete", data = "<req>")]
pub async fn delete(req: Json<IdReq>, _user: UserPrincipal) -> Res<()> {
    match service::delete(req.into_inner()).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 添加模型提供商
#[post("/provider/add", data = "<req>")]
pub async fn add_provider(req: Json<ProviderAddReq>, user: UserPrincipal) -> Res<()> {
    match service::add_provider(req.into_inner(), user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 修改模型提供商
#[post("/provider/update", data = "<req>")]
pub async fn update_provider(req: Json<ProviderUpdateReq>, user: UserPrincipal) -> Res<()> {
    match service::update_provider(req.into_inner(), user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 删除模型提供商
#[post("/provider/delete", data = "<req>")]
pub async fn delete_provider(req: Json<IdReq>, _user: UserPrincipal) -> Res<()> {
    match service::delete_provider(req.into_inner()).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

// -- 用量统计 --

#[post("/usage/overview")]
pub async fn usage_overview(_user: UserPrincipal) -> Res<UsageOverview> {
    match usage::overview().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/usage/trend", data = "<req>")]
pub async fn usage_trend(req: Json<TrendReq>, _user: UserPrincipal) -> Res<Vec<TrendItem>> {
    match usage::trend(req.into_inner()).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/usage/model_rank", data = "<req>")]
pub async fn usage_model_rank(req: Json<RankReq>, _user: UserPrincipal) -> Res<Vec<ModelRankItem>> {
    match usage::model_rank(req.into_inner()).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
