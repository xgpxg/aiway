use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct McpServer {
    pub name: String,
    pub description: Option<String>,
    pub server_type: McpServerType,
    pub tools: HashMap<String, McpTool>,
    pub proxy_config: Option<McpServerProxyConfig>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[non_exhaustive]
pub enum McpServerType {
    #[default]
    /// HTTP接口适配
    Http,
    /// 代理直连
    Proxy,
    // /// 数据库
    //Database,
}

/// MCP服务代理配置
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct McpServerProxyConfig {
    /// 代理URL，目前仅支持 Streamable HTTP
    pub url: String,
    /// 请求头
    pub headers: HashMap<String, String>,
}

/// MCP工具定义
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct McpTool {
    /// 工具名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 输入参数
    pub input_schema: Option<Value>,
    /// 路由配置
    pub route: Route,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct Route {
    pub route_type: RouteType,
    /// 请求的服务，可自动负载均衡
    pub service_name: Option<String>,
    /// 服务路径
    pub service_path: Option<String>,
    /// 请求地址，可选
    pub url: Option<String>,
    /// 请求方法
    pub method: Option<String>,
    /// 请求参数配置
    pub request_param: Option<Value>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[non_exhaustive]
pub enum RouteType {
    /// 无路由。当Server类型为 [McpServerType::Proxy] 时，无需路由。
    #[default]
    NoRoute,
    /// 已有服务的服务
    Service,
    /// 指定URL
    Url,
}
