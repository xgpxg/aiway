use busi::res::Res;
use rocket::{get, routes};
use crate::server::mcp_proxy::mcp;

pub fn routes() -> Vec<rocket::Route> {
    routes![all_mcp_servers]
}

/// 获取所有MCP Server
#[get("/mcp/servers")]
async fn all_mcp_servers() -> Res<Vec<aiway_protocol::mcp::mcp::McpServer>> {
    match mcp::all_mcp_servers().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
