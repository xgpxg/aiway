use crate::server::auth::UserPrincipal;
use crate::server::route::request::{
    RouteAddOrUpdateReq, RouteListReq, UpdateGlobalFilterConfigReq, UpdateStatusReq,
};
use crate::server::route::response::RouteListRes;
use crate::server::route::service;
use aiway_protocol::common::req::IdsReq;
use aiway_protocol::common::res::{PageRes, Res};
use aiway_protocol::gateway::GlobalFilter;
use rocket::serde::json::Json;
use rocket::{post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        add,
        list,
        update,
        delete,
        update_status,
        update_global_filter_config,
        get_global_filter_config
    ]
}

/// 添加路由
#[post("/add", data = "<req>")]
pub async fn add(req: Json<RouteAddOrUpdateReq>, user: UserPrincipal) -> Res<()> {
    match service::add(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 路由列表
#[post("/list", data = "<req>")]
pub async fn list(req: Json<RouteListReq>, user: UserPrincipal) -> Res<PageRes<RouteListRes>> {
    match service::list(req.0, user).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/update", data = "<req>")]
pub async fn update(req: Json<RouteAddOrUpdateReq>, user: UserPrincipal) -> Res<()> {
    match service::update(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}
/// 删除路由
#[post("/delete", data = "<req>")]
pub async fn delete(req: Json<IdsReq>, user: UserPrincipal) -> Res<()> {
    match service::delete(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 更新状态
#[post("/update_status", data = "<req>")]
pub async fn update_status(req: Json<UpdateStatusReq>, user: UserPrincipal) -> Res<()> {
    match service::update_status(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/global_filter/update", data = "<req>")]
pub async fn update_global_filter_config(
    req: Json<UpdateGlobalFilterConfigReq>,
    user: UserPrincipal,
) -> Res<()> {
    match service::update_global_filter_config(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/global_filter")]
pub async fn get_global_filter_config(user: UserPrincipal) -> Res<GlobalFilter> {
    match service::get_global_filter_config(user).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
