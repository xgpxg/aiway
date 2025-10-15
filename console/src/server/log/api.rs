use crate::server::auth::UserPrincipal;
use crate::server::log::request::LogListReq;
use crate::server::log::response::LogListRes;
use crate::server::log::service;
use protocol::common::res::{PageRes, Res};
use rocket::{post, routes};
use rocket::serde::json::Json;

pub fn routes() -> Vec<rocket::Route> {
    routes![list]
}

/// 查询日志
#[post("/list", data = "<req>")]
pub async fn list(req: Json<LogListReq>, _user: UserPrincipal) -> Res<PageRes<LogListRes>> {
    match service::list(req.0).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
