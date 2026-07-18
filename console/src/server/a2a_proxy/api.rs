use aiway_protocol::a2a::Agent;
use busi::res::Res;
use rocket::{get, routes};

use crate::server::a2a_proxy::agent;

pub fn routes() -> Vec<rocket::Route> {
    routes![all_agents]
}

/// 获取所有启用的 Agent（供网关拉取）
#[get("/a2a/agents")]
async fn all_agents() -> Res<Vec<Agent>> {
    match agent::all_agents().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
