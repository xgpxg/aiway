use derive_builder::Builder;
use rbatis::crud;
use rbatis::rbdc::DateTime;
use rocket::serde::{Deserialize, Serialize};

/// A2A Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct Agent {
    pub id: Option<i64>,
    /// Agent 名称，全局唯一，用于路由：/v1/a2a/<name>/
    pub name: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// Agent 端点 URL
    pub url: Option<String>,
    /// 状态：Disable | Ok
    pub status: Option<AgentStatus>,
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
pub enum AgentStatus {
    /// 停用
    #[default]
    Disable,
    /// 启用
    Ok,
}

crud!(Agent {});
