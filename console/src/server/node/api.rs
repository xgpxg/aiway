use crate::server::auth::UserPrincipal;
use crate::server::node::request::GatewayNodeListReq;
use crate::server::node::response::GatewayNodeListRes;
use crate::server::node::service;
use busi::res::{PageRes, Res};
use rocket::serde::json::Json;
use rocket::{ post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![list]
}

/// 网关节点列表
#[post("/list", data = "<req>")]
pub async fn list(
    req: Json<GatewayNodeListReq>,
    _user: UserPrincipal,
) -> Res<PageRes<GatewayNodeListRes>> {
    match service::list(req.0).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
