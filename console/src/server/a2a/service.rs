use crate::server::a2a::request::{
    AgentAddReq, AgentListReq, AgentStatusUpdateReq, AgentUpdateReq,
};
use crate::server::a2a::response::AgentListRes;
use crate::server::auth::UserPrincipal;
use crate::server::db::models::agent::{Agent, AgentBuilder, AgentStatus};
use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use crate::server::db::{Pool, tools};
use anyhow::bail;
use busi::req::IdsReq;
use common::id;
use rbs::value;
use serde_json;

pub async fn agent_add(req: AgentAddReq, user: UserPrincipal) -> anyhow::Result<()> {
    let agent = AgentBuilder::default()
        .id(id::next().into())
        .name(req.name.into())
        .description(req.description)
        .url(req.url.into())
        .status(AgentStatus::Disable.into())
        .create_user_id(Some(user.id))
        .create_time(Some(tools::now()))
        .build()?;

    if check_agent_exists(&agent, None).await? {
        bail!(
            "Agent with name {} already exists",
            agent.name.unwrap()
        )
    }
    Agent::insert(Pool::get()?, &agent).await?;
    Ok(())
}

async fn check_agent_exists(agent: &Agent, exclude_id: Option<i64>) -> anyhow::Result<bool> {
    let mut list = Agent::select_by_map(
        Pool::get()?,
        value! {
            "name": &agent.name,
        },
    )
    .await?;

    list.retain(|item| item.id != exclude_id);

    Ok(!list.is_empty())
}

pub async fn agent_list(_req: AgentListReq) -> anyhow::Result<Vec<AgentListRes>> {
    let mut list = Agent::select_all(Pool::get()?).await?;
    list.sort_by(|a, b| b.id.cmp(&a.id));

    let list = list
        .into_iter()
        .map(|item| AgentListRes { inner: item })
        .collect::<Vec<_>>();

    Ok(list)
}

pub async fn agent_update(req: AgentUpdateReq, user: UserPrincipal) -> anyhow::Result<()> {
    let old = Agent::select_by_map(Pool::get()?, value! { "id": req.id }).await?;
    if old.is_empty() {
        bail!("Agent not found");
    }

    let old = old.first().unwrap();

    if check_agent_exists(old, Some(req.id)).await? {
        bail!(
            "Agent with name {} already exists",
            old.name.as_ref().unwrap()
        )
    }

    let update = AgentBuilder::default()
        .id(req.id.into())
        .name(req.name)
        .description(req.description)
        .url(req.url)
        .update_user_id(Some(user.id))
        .update_time(Some(tools::now()))
        .build()?;

    Agent::update_by_map(Pool::get()?, &update, value! { "id": req.id }).await?;
    Ok(())
}

pub async fn agent_update_status(
    req: AgentStatusUpdateReq,
    user: UserPrincipal,
) -> anyhow::Result<()> {
    let old = Agent::select_by_map(Pool::get()?, value! { "id": req.id }).await?;
    if old.is_empty() {
        bail!("Agent not found")
    }
    Agent::update_by_map(
        Pool::get()?,
        &AgentBuilder::default()
            .id(Some(req.id))
            .status(Some(req.status))
            .update_user_id(Some(user.id))
            .update_time(Some(tools::now()))
            .build()?,
        value! { "id": req.id },
    )
    .await?;
    Ok(())
}

pub async fn agent_delete(req: IdsReq) -> anyhow::Result<()> {
    Agent::delete_by_map(Pool::get()?, value! { "id": req.ids }).await?;
    Ok(())
}

/// 获取 Agent Card 并重写 url 字段为网关入口地址
///
/// 流程：
/// 1. 从 DB 查询 Agent（获取 name 和 url）
/// 2. 请求真实 Agent 的 `/.well-known/agent-card.json`
/// 3. 将返回 JSON 中的 `url` 重写为 `{agent_entry_url}/v1/a2a/{agent-name}`
/// 4. 返回修改后的 Agent Card JSON
pub async fn fetch_agent_card(id: i64) -> anyhow::Result<serde_json::Value> {
    let agents = Agent::select_by_map(Pool::get()?, value! { "id": id }).await?;
    if agents.is_empty() {
        bail!("Agent not found");
    }
    let agent = agents.first().unwrap();
    let agent_name = agent.name.as_ref().ok_or_else(|| anyhow::anyhow!("Agent name is missing"))?;
    let agent_url = agent.url.as_ref().ok_or_else(|| anyhow::anyhow!("Agent url is missing"))?;

    // 获取配置的 Agent 服务入口地址
    let entry_url: String = SystemConfig::get(ConfigKey::AgentEntryUrl).await?;
    if entry_url.is_empty() {
        bail!("Agent entry URL not configured. Please set it in system settings.");
    }

    // 请求真实 Agent 的 Agent Card
    let card_url = format!(
        "{}/.well-known/agent-card.json",
        agent_url.trim_end_matches('/')
    );
    let resp = reqwest::Client::new()
        .get(&card_url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    if !resp.status().is_success() {
        bail!(
            "Failed to fetch agent card: status {}",
            resp.status()
        );
    }

    let body = resp.text().await?;
    let mut card: serde_json::Value = serde_json::from_str(&body)?;

    // 重写 url 为网关入口地址，避免暴露后端内部地址
    let gateway_agent_url = format!(
        "{}/v1/a2a/{}",
        entry_url.trim_end_matches('/'),
        agent_name
    );
    card["url"] = serde_json::Value::String(gateway_agent_url);

    Ok(card)
}
