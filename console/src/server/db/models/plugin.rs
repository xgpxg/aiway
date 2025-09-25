use derive_builder::Builder;
use protocol::gateway::plugin::PluginPhase;
use protocol::gateway::service::LbStrategy;
use rbatis::crud;
use rbatis::rbdc::DateTime;
use rocket::serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// 路由配置
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct Plugin {
    pub id: Option<i64>,
    /// 插件名称
    pub name: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 服务节点，JSON数组，支持IP和域名，如["http://127.0.0.1:8080"]
    pub phase: Option<PluginPhase>,
    /// 下载地址
    pub url: Option<String>,
    /// 版本
    pub version: Option<String>,
    /// 创建人ID
    pub create_user_id: Option<i64>,
    /// 修改人ID
    pub update_user_id: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 备注
    pub remark: Option<String>,
    /// 是否删除
    pub is_delete: Option<i8>,
}

crud!(Plugin {});
