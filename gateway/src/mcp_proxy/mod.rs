mod components;
mod proxy;

pub(crate) use proxy::mcp_get as mcp_get_endpoint;
pub(crate) use proxy::mcp_post as mcp_post_endpoint;
use crate::Args;

pub async fn init(_args: &Args) {
    components::McpFactory::init().await;
}