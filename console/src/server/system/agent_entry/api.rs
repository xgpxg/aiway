use crate::server::auth::UserPrincipal;
use crate::server::system::agent_entry::request::AgentEntryConfig;
use crate::server::system::agent_entry::service;
use busi::res::Res;
use rocket::serde::json::Json;
use rocket::{get, post};

/// 更新 Agent 服务入口地址
#[post("/agent_entry/update", data = "<req>")]
pub async fn update(req: Json<AgentEntryConfig>, _user: UserPrincipal) -> Res<()> {
    match service::update(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 获取 Agent 服务入口地址
#[get("/agent_entry")]
pub async fn get(_user: UserPrincipal) -> Res<AgentEntryConfig> {
    match service::get().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
