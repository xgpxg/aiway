use rocket::serde::{Deserialize, Serialize};
use crate::server::db::models::mcp_server::McpServer;
use crate::server::db::models::mcp_tool::McpTool;

#[derive(Debug, Serialize, Deserialize)]
pub struct McpServerListRes {
    #[serde(flatten)]
    pub inner: McpServer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpToolListRes {
    #[serde(flatten)]
    pub inner: McpTool,
}
