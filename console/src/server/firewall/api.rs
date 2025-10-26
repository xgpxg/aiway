use crate::server::auth::UserPrincipal;
use crate::server::firewall::request::FirewallUpdateReq;
use crate::server::firewall::response::DemoRes;
use crate::server::firewall::service;
use protocol::common::res::{PageRes, Res};
use protocol::gateway::Firewall;
use rocket::serde::json::Json;
use rocket::{get, post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![update, detail]
}

/// 更新防火墙配置
#[post("/update", data = "<req>")]
pub async fn update(req: Json<FirewallUpdateReq>, _user: UserPrincipal) -> Res<()> {
    match service::update(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 防火墙配置详情
#[get("/detail")]
pub async fn detail(_user: UserPrincipal) -> Res<Firewall> {
    match service::detail().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
