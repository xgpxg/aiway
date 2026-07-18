use crate::server::db::models::agent::AgentStatus;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAddReq {
    /// Agent 名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// Agent 端点 URL
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUpdateReq {
    pub id: i64,
    /// Agent 名称
    pub name: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// Agent 端点 URL
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatusUpdateReq {
    pub id: i64,
    pub status: AgentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentListReq {
    /// 模糊搜索：名称/描述
    pub filter_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCardReq {
    pub id: i64,
}
