use aiway_protocol::mcp::mcp::{McpServerProxyConfig, McpServerType};
use derive_builder::Builder;
use rbatis::crud;
use rbatis::rbdc::DateTime;
use rocket::serde::{Deserialize, Serialize};

/// MCP 服务配置
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct McpServer {
    pub id: Option<i64>,
    /// 服务名称
    pub name: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 服务类型
    pub server_type: Option<McpServerType>,
    /// 代理配置
    pub proxy_config: Option<McpServerProxyConfig>,
    /// 状态：Disable | Ok
    pub status: Option<McpServerStatus>,
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
pub enum McpServerStatus {
    /// 停用
    #[default]
    Disable,
    /// 启用
    Ok,
}

crud!(McpServer {});
