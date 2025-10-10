use crate::server::plugin::PluginListReq;
use derive_builder::Builder;
use rbatis::rbdc::DateTime;
use rbatis::{crud, htmlsql_select_page};
use rocket::serde::{Deserialize, Serialize};

/// 路由配置
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct Plugin {
    pub id: Option<i64>,
    /// 插件名称
    pub name: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 下载地址，该地址用于gateway下载插件，需保证从gateway处可以访问。
    pub url: Option<String>,
    /// 版本，格式为0.1.0，只增不减
    pub version: Option<String>,
    /// 默认配置，JSON格式
    pub default_config: Option<serde_json::Value>,
    /// 创建人ID
    pub create_user_id: Option<i64>,
    /// 修改人ID
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

crud!(Plugin {});
htmlsql_select_page!(list_page(param: &PluginListReq) -> Plugin => "src/server/db/mapper/plugin.html");
