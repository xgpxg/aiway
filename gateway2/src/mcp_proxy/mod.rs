mod components;
mod proxy;

use pingora::Error;
use pingora_proxy::Session;
pub(crate) use proxy::mcp_post as mcp_post_endpoint;
use crate::Args;

pub async fn init(_args: &Args) {
    components::McpFactory::init().await;
}

/// 处理 MCP 请求
pub(crate) async fn handle_mcp_request(
    session: &mut Session,
    path: &str,
) -> pingora::Result<bool, Box<Error>> {
    let body = session.read_request_body().await?.unwrap_or_default();
    let response = mcp_post_endpoint(path, body).await.unwrap();
    crate::service::forward_reqwest_to_pingora(response, session).await?;
    Ok(true)
}
