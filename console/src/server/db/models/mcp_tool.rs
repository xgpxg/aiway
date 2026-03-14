use crate::server::mcp::McpToolListReq;
use derive_builder::Builder;
use rbatis::rbdc::DateTime;
use rbatis::{crud, htmlsql_select_page};
use rocket::serde::{Deserialize, Serialize};
use serde_json::Value;
use aiway_protocol::mcp::mcp::RouteType;

/// MCP 工具配置
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct McpTool {
    pub id: Option<i64>,
    /// MCP 服务 ID
    pub mcp_server_id: Option<i64>,
    /// 工具名称
    pub name: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 输入参数 Schema
    pub input_schema: Option<Value>,
    /// 输出参数 Schema
    pub output_schema: Option<Value>,
    /// 路由类型
    pub route_type: Option<RouteType>,
    /// 目标服务名称，可自动负载均衡
    pub service_name: Option<String>,
    /// 服务路径
    pub service_path: Option<String>,
    /// 目标服务地址
    pub url: Option<String>,
    /// 请求方法
    pub method: Option<String>,
    /// 请求参数配置
    pub request_param: Option<Value>,
    /// 状态：Disable | Ok
    pub status: Option<McpToolStatus>,
    /// 创建人 ID
    pub create_user_id: Option<i64>,
    /// 修改人 ID
    pub update_user_id: Option<i64>,
    /// 创建时间
    #[serde(serialize_with = "crate::server::common::serialize_datetime")]
    pub create_time: Option<DateTime>,
    /// 更新时间
    #[serde(serialize_with = "crate::server::common::serialize_datetime")]
    pub update_time: Option<DateTime>,
    /// 备注
    pub remark: Option<String>,
    /// 是否删除
    pub is_delete: Option<i8>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum McpToolStatus {
    /// 停用
    #[default]
    Disable,
    /// 启用
    Ok,
}



crud!(McpTool {});

htmlsql_select_page!(list_page(param: &McpToolListReq) -> McpTool => "src/server/db/mapper/mcp_tool.html");
