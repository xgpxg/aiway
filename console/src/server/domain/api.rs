use crate::server::auth::UserPrincipal;
use crate::server::domain::request::{DomainAddOrUpdateReq, UpdateStatusReq};
use crate::server::domain::response::DomainListRes;
use crate::server::domain::{service, DomainListReq};
use busi::req::IdsReq;
use busi::res::{PageRes, Res};
use rocket::serde::json::Json;
use rocket::{post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![add, update, update_status, delete, list]
}

/// 添加域名
#[post("/add", data = "<req>")]
pub async fn add(req: Json<DomainAddOrUpdateReq>, user: UserPrincipal) -> Res<()> {
    match service::add(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 更新域名
#[post("/update", data = "<req>")]
pub async fn update(req: Json<DomainAddOrUpdateReq>, user: UserPrincipal) -> Res<()> {
    match service::update(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 删除域名
#[post("/delete", data = "<req>")]
pub async fn delete(req: Json<IdsReq>, _user: UserPrincipal) -> Res<()> {
    match service::delete(req.0).await {
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

/// 域名列表
#[post("/list", data = "<req>")]
pub async fn list(req: Json<DomainListReq>, _user: UserPrincipal) -> Res<PageRes<DomainListRes>> {
    match service::list(req.0).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
