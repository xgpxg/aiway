use crate::server::db::models::mcp_server::McpServerStatus;
use crate::server::db::models::mcp_tool::McpToolStatus;
use aiway_protocol::mcp::mcp::{McpServerProxyConfig, McpServerType, RouteType};
use busi::impl_pagination;
use busi::req::PageReq;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerAddReq {
    /// 服务名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 服务类型
    pub server_type: McpServerType,
    /// 代理配置
    pub proxy_config: Option<McpServerProxyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerUpdateReq {
    pub id: i64,
    pub name: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 服务类型
    pub server_type: Option<McpServerType>,
    /// 代理配置
    pub proxy_config: Option<McpServerProxyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerStatusUpdateReq {
    pub id: i64,
    pub status: McpServerStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerListReq {
    /// 模糊搜索：服务名/描述
    pub filter_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolAddReq {
    pub mcp_server_id: i64,
    /// 工具名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 输入参数 Schema
    pub input_schema: Option<serde_json::Value>,
    /// 输出参数 Schema
    pub output_schema: Option<serde_json::Value>,
    /// 路由类型
    pub route_type: Option<RouteType>,
    /// 目标服务名称
    pub service_name: Option<String>,
    /// 目标服务路径
    pub service_path: Option<String>,
    /// 目标服务地址
    pub url: Option<String>,
    /// 请求方法
    pub method: Option<String>,
    /// 请求参数配置
    pub request_param: Option<serde_json::Value>,
}

fn default_mcp_tool_status() -> McpToolStatus {
    McpToolStatus::Disable
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolUpdateReq {
    pub id: i64,
    /// 描述
    pub description: Option<String>,
    /// 输入参数 Schema
    pub input_schema: Option<serde_json::Value>,
    /// 输出参数 Schema
    pub output_schema: Option<serde_json::Value>,
    /// 路由类型
    pub route_type: Option<RouteType>,
    /// 目标服务名称
    pub service_name: Option<String>,
    /// 目标服务路径
    pub service_path: Option<String>,
    /// 目标服务地址
    pub url: Option<String>,
    /// 请求方法
    pub method: Option<String>,
    /// 请求参数配置
    pub request_param: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolListReq {
    pub page: PageReq,
    /// 所属服务ID
    pub mcp_server_id: i64,
    /// 模糊搜索：工具名/描述
    pub filter_text: Option<String>,
    /// 状态
    pub status: Option<McpToolStatus>,
    /// 所属服务
    pub service: Option<String>,
}
impl_pagination!(McpToolListReq);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMcpToolStatusReq {
    pub id: i64,
    pub status: McpToolStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProxyServerToolsReq {
    pub mcp_server_id: i64,
    /// 描述
    pub url: Option<String>,
    /// 输入参数 Schema
    pub headers: Option<serde_json::Value>,
}
