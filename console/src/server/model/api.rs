use crate::server::auth::UserPrincipal;
use crate::server::model::request::{
    ModelAddReq, ModelLisReq, ModelUpdateReq, ProviderAddReq, ProviderUpdateReq,
};
use crate::server::model::response::ModelListRes;
use crate::server::model::service;
use aiway_protocol::common::req::IdReq;
use aiway_protocol::common::res::Res;
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
        delete_provider
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
