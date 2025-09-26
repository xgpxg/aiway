use rocket::{post, routes};
use rocket::serde::json::Json;
use protocol::common::res::Res;
use crate::server::auth::UserPrincipal;
use crate::server::key::request::ApiKeyAddOrUpdateReq;
use crate::server::key::service;

pub fn routes() -> Vec<rocket::Route> {
    routes![add]
}

/// 添加路由
#[post("/add", data = "<req>")]
pub async fn add(req: Json<ApiKeyAddOrUpdateReq>, user: UserPrincipal) -> Res<()> {
    match service::add(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}