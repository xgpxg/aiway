use crate::server::auth::UserPrincipal;
use crate::server::route::request::RouteAddReq;
use crate::server::route::service;
use protocol::common::res::Res;
use rocket::serde::json::Json;
use rocket::{post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![add]
}

/// 添加路由
#[post("/add", data = "<req>")]
pub async fn add(req: Json<RouteAddReq>, user: UserPrincipal) -> Res<()> {
    match service::add(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}
