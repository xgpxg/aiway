use crate::server::key::ApiKeyListReq;
use derive_builder::Builder;
use rbatis::executor::Executor;
use rbatis::rbdc::DateTime;
use rbatis::{crud, htmlsql, htmlsql_select_page};
use rocket::serde::{Deserialize, Serialize};

/// 路由配置
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct ApiKey {
    pub id: Option<i64>,
    /// 密钥名称
    pub name: Option<String>,
    /// 密钥所属的主体标识，可以为空
    pub principal: Option<String>,
    /// 密钥
    pub secret: Option<String>,
    /// 状态，0禁用 1启用
    pub status: Option<ApiKeyStatus>,
    /// 生效时间
    #[serde(serialize_with = "crate::server::common::serialize_datetime")]
    pub eff_time: Option<DateTime>,
    /// 到期时间
    #[serde(serialize_with = "crate::server::common::serialize_datetime")]
    pub exp_time: Option<DateTime>,
    /// 来源
    pub source: Option<ApiKeySource>,
    /// 密钥变更时间戳，UTC，毫秒级，用于数据同步。
    /// 重要：密钥新增、修改、删除都需要更新该字段。
    pub ts: Option<i64>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ApiKeyStatus {
    /// 禁用
    #[default]
    Disable,
    /// 启用
    Ok,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ApiKeySource {
    /// 控制台
    #[default]
    Console,
}

crud!(ApiKey {});
htmlsql_select_page!(list_page(param: &ApiKeyListReq) -> ApiKey => "src/server/db/mapper/api_key.html");
htmlsql!(list_by_update_time(rb: &dyn Executor,update_time: i64) -> Vec<ApiKey> => "src/server/db/mapper/api_key.html");
