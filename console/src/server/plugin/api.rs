use crate::server::auth::UserPrincipal;
use crate::server::plugin::request::{PluginAddReq, PluginInfoReq, PluginListReq, PluginUpdateReq};
use crate::server::plugin::response::{PluginInfoRes, PluginListRes};
use crate::server::plugin::service;
use protocol::common::req::IdsReq;
use protocol::common::res::{PageRes, Res};
use rocket::form::Form;
use rocket::serde::json::Json;
use rocket::{post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![info, add, delete, list, update]
}

/// 解析插件信息
#[post("/info", data = "<req>")]
async fn info(req: Form<PluginInfoReq<'_>>, user: UserPrincipal) -> Res<PluginInfoRes> {
    match service::info(req.into_inner(), user).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 新增插件
#[post("/add", data = "<req>")]
async fn add(req: Form<PluginAddReq<'_>>, user: UserPrincipal) -> Res<()> {
    match service::add(req.into_inner(), user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 删除插件
#[post("/delete", data = "<req>")]
async fn delete(req: Json<IdsReq>, _user: UserPrincipal) -> Res<()> {
    match service::delete(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 插件列表
#[post("/list", data = "<req>")]
async fn list(req: Json<PluginListReq>, _user: UserPrincipal) -> Res<PageRes<PluginListRes>> {
    match service::list(req.0).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 更新插件
#[post("/update", data = "<req>")]
async fn update(req: Form<PluginUpdateReq<'_>>, user: UserPrincipal) -> Res<()> {
    match service::update(req.into_inner(), user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}
