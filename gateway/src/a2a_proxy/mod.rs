//! # A2A 协议代理
//!
//! 作为 A2A 协议的透明代理（Agent Broker），将请求按 Agent 名称路由到后端 Agent 端点。
//! 不做协议内容转换，仅负责路由、转发、流式回传。

mod components;

use crate::Args;
use aiway_protocol::common::constants::A2A_API_PREFIX;
use bytes::Bytes;
use pingora::http::ResponseHeader;
use pingora::prelude::Session;
use pingora::Error;
use reqwest::{Client, ClientBuilder};
use serde_json;
use std::sync::LazyLock;
use std::time::Duration;

pub use components::AgentFactory;

/// A2A 代理专用的 HTTP 客户端（支持流式响应）
static A2A_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    ClientBuilder::default()
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(300))
        .build()
        .unwrap()
});

pub async fn init(_args: &Args) {
    AgentFactory::init().await;
}

/// 处理 A2A 请求
///
/// 路径格式：`/v1/a2a/<agent-name>/<rest-path>`
///
/// 1. 从路径中提取 agent 名称
/// 2. 查找 Agent 端点 URL
/// 3. 透传请求体到 Agent
/// 4. 流式回传响应（agent-card 请求特殊处理：GET + URL 重写）
pub(crate) async fn handle_a2a_request(
    session: &mut Session,
    path: &str,
) -> pingora::Result<bool, Box<Error>> {
    // 解析 agent 名称：/v1/a2a/<agent-name>/...
    let agent_name = extract_agent_name(path).ok_or_else(|| {
        pingora::Error::new_str("invalid a2a path: missing agent name")
    })?;

    log::info!("[A2A] Routing request to agent: {}", agent_name);

    // 查找 Agent
    let agent = AgentFactory::get_agent(agent_name).ok_or_else(|| {
        log::error!("[A2A] Agent not found: {}", agent_name);
        pingora::Error::new_str("agent not found")
    })?;

    // 构建目标 URL（去掉 /v1/a2a/<agent-name> 前缀，保留剩余路径）
    let rest = &path[A2A_API_PREFIX.len() + agent_name.len()..];
    let target_url = format!("{}{}", agent.url.trim_end_matches('/'), rest);

    // Agent Card 发现请求：GET，缓冲响应并重写 url 字段
    if is_agent_card_request(rest) {
        return handle_agent_card(session, &target_url, path).await;
    }

    // 普通任务请求：POST 透传
    let body = session.read_request_body().await?.unwrap_or_default();
    let response = forward_to_agent(session, &target_url, body).await?;
    crate::service::forward_reqwest_to_pingora(response, session).await?;

    Ok(true)
}

/// 判断请求是否为 Agent Card 发现请求
///
/// 约定路径：`/.well-known/agent-card.json`（GET 请求）
fn is_agent_card_request(rest: &str) -> bool {
    rest == "/.well-known/agent-card.json" || rest == "/.well-known/agent.json"
}

/// 处理 Agent Card 发现请求
///
/// 向真实 Agent 发起 GET 请求获取 Agent Card，
/// 将响应中的 `url` 字段重写为网关路由地址，避免暴露后端真实地址。
///
/// 重写示例：
/// - 原始 `url`: `http://10.0.0.5:8080/a2a`
/// - 重写为 `http://<gateway>/v1/a2a/<agent-name>`
async fn handle_agent_card(
    session: &mut Session,
    target_url: &str,
    path: &str,
) -> pingora::Result<bool, Box<Error>> {
    let response = A2A_CLIENT
        .get(target_url)
        .send()
        .await
        .map_err(|e| {
            log::error!("[A2A] Failed to fetch agent card: {}", e);
            pingora::Error::new_str("failed to fetch agent card from backend")
        })?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| {
            log::error!("[A2A] Failed to read agent card body: {}", e);
            pingora::Error::new_str("failed to read agent card body")
        })?;

    // 解析并重写 url 字段
    let mut card: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
        log::error!("[A2A] Invalid agent card JSON: {}", e);
        pingora::Error::new_str("invalid agent card JSON")
    })?;

    // 重写 url 为网关路由地址，避免暴露后端内部地址
    let host = session.req_header()
        .headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:7001");
    let gateway_agent_url = format!(
        "http://{}{}",
        host,
        path.trim_end_matches('/')
    );
    card["url"] = serde_json::Value::String(gateway_agent_url);

    let new_body = serde_json::to_vec(&card).map_err(|e| {
        log::error!("[A2A] Failed to serialize modified agent card: {}", e);
        pingora::Error::new_str("failed to serialize agent card")
    })?;

    // 构建响应头（使用修改后的 content-length）
    let mut header = ResponseHeader::build(status, None)?;
    header.insert_header("content-type", "application/json")?;
    header.insert_header("content-length", new_body.len().to_string())?;

    session.write_response_header(Box::new(header), false).await?;
    session.write_response_body(Some(Bytes::from(new_body)), true).await?;

    Ok(true)
}

/// 从路径中提取 Agent 名称
///
/// `/v1/a2a/my-agent/...` → `Some("my-agent")`
fn extract_agent_name(path: &str) -> Option<&str> {
    let after_prefix = &path[A2A_API_PREFIX.len()..];
    // 取到下一个 `/` 或末尾
    let end = after_prefix.find('/').unwrap_or(after_prefix.len());
    if end == 0 { None } else { Some(&after_prefix[..end]) }
}

/// 将请求转发到 Agent 端点
async fn forward_to_agent(
    session: &Session,
    target_url: &str,
    body: Bytes,
) -> pingora::Result<reqwest::Response, Box<Error>> {
    let mut request_builder = A2A_CLIENT.post(target_url).body(body);

    // 透传请求头（排除被禁止的头部）
    for (name, value) in session.req_header().headers.iter() {
        let name_str = name.as_str();
        if aiway_protocol::common::constants::BAN_HEADERS.contains(name_str) {
            continue;
        }
        request_builder = request_builder.header(name_str, value.as_bytes());
    }

    request_builder
        .send()
        .await
        .map_err(|e| {
            log::error!("[A2A] Forward request failed: {}", e);
            pingora::Error::new_str("failed to forward request to agent")
        })
}
