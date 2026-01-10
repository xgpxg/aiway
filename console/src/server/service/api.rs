use crate::server::auth::UserPrincipal;
use crate::server::service::request::{
    ServiceAddReq, ServiceListReq, ServiceUpdateReq, UpdateStatusReq,
};
use crate::server::service::response::ServiceListRes;
use crate::server::service::service;
use aiway_protocol::common::req::IdsReq;
use aiway_protocol::common::res::{PageRes, Res};
use rocket::serde::json::Json;
use rocket::{post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![add, list, update, delete, update_status]
}

#[post("/add", data = "<req>")]
async fn add(req: Json<ServiceAddReq>, user: UserPrincipal) -> Res<()> {
    match service::add(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/list", data = "<req>")]
async fn list(req: Json<ServiceListReq>, _user: UserPrincipal) -> Res<PageRes<ServiceListRes>> {
    match service::list(req.0).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/update", data = "<req>")]
async fn update(req: Json<ServiceUpdateReq>, user: UserPrincipal) -> Res<()> {
    match service::update(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/delete", data = "<req>")]
async fn delete(req: Json<IdsReq>, _user: UserPrincipal) -> Res<()> {
    match service::delete(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/update_status", data = "<req>")]
pub async fn update_status(req: Json<UpdateStatusReq>, user: UserPrincipal) -> Res<()> {
    match service::update_status(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}
