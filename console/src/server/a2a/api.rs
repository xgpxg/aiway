use crate::server::a2a::request::{
    AgentAddReq, AgentCardReq, AgentListReq, AgentStatusUpdateReq, AgentUpdateReq,
};
use crate::server::a2a::response::AgentListRes;
use crate::server::a2a::service;
use crate::server::auth::UserPrincipal;
use busi::req::IdsReq;
use busi::res::Res;
use rocket::serde::json::Json;
use rocket::{post, routes};
use serde_json;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        agent_add,
        agent_list,
        agent_update,
        agent_update_status,
        agent_delete,
        agent_card,
    ]
}

#[post("/agent/add", data = "<req>")]
async fn agent_add(req: Json<AgentAddReq>, user: UserPrincipal) -> Res<()> {
    match service::agent_add(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/agent/list", data = "<req>")]
async fn agent_list(req: Json<AgentListReq>, _user: UserPrincipal) -> Res<Vec<AgentListRes>> {
    match service::agent_list(req.0).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/agent/update", data = "<req>")]
async fn agent_update(req: Json<AgentUpdateReq>, user: UserPrincipal) -> Res<()> {
    match service::agent_update(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/agent/update_status", data = "<req>")]
async fn agent_update_status(req: Json<AgentStatusUpdateReq>, user: UserPrincipal) -> Res<()> {
    match service::agent_update_status(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/agent/delete", data = "<req>")]
async fn agent_delete(req: Json<IdsReq>, _user: UserPrincipal) -> Res<()> {
    match service::agent_delete(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 获取 Agent Card（管理员验证用，url 字段已重写为网关入口地址）
#[post("/agent/card", data = "<req>")]
async fn agent_card(
    req: Json<AgentCardReq>,
    _user: UserPrincipal,
) -> Res<serde_json::Value> {
    match service::fetch_agent_card(req.id).await {
        Ok(card) => Res::success(card),
        Err(e) => Res::error(&e.to_string()),
    }
}
