use crate::server::db::Pool;
use crate::server::db::models::agent::{Agent, AgentStatus};
use aiway_protocol::a2a::Agent as AgentProtocol;
use rbs::value;

/// 查询所有启用的 Agent，转为协议层类型供网关拉取
pub(crate) async fn all_agents() -> anyhow::Result<Vec<AgentProtocol>> {
    let agents = Agent::select_by_map(
        Pool::get()?,
        value! {
            "status": AgentStatus::Ok
        },
    )
    .await?;

    let list = agents
        .into_iter()
        .map(|a| AgentProtocol {
            name: a.name.unwrap(),
            description: a.description,
            url: a.url.unwrap(),
        })
        .collect();

    Ok(list)
}
