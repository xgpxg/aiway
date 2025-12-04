use crate::server::route::RouteListReq;
use derive_builder::Builder;
use protocol::gateway::plugin::ConfiguredPlugin;
use rbatis::rbdc::DateTime;
use rbatis::{crud, htmlsql_select_page};
use rocket::serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 路由配置
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct Route {
    pub id: Option<i64>,
    /// 路由名称
    pub name: Option<String>,
    /// 路由描述
    pub description: Option<String>,
    /// 状态：0停用 1启用
    pub status: Option<RouteStatus>,
    /// 需要匹配的域名
    pub host: Option<String>,
    // /// 路由前缀
    // pub prefix: Option<String>,
    /// 路由路径
    pub path: Option<String>,
    // /// 是否去除路径前缀
    // pub strip_prefix: Option<i8>,
    /// 目标服务名称
    pub service: Option<String>,
    // 请求协议：http | https
    //pub protocol: Option<String>,
    // 请求方法：GET | POST | PUT | DELETE | HEAD | OPTIONS | PATCH | TRACE | CONNECT
    //pub method: Option<String>,
    /// 按请求头匹配
    pub header: Option<BTreeMap<String, String>>,
    /// 按请求参数匹配
    pub query: Option<BTreeMap<String, String>>,
    /// 请求阶段过滤器，JSON数组
    pub pre_filters: Option<Vec<ConfiguredPlugin>>,
    /// 响应阶段过滤器，JSON数组
    pub post_filters: Option<Vec<ConfiguredPlugin>>,
    /// 是否开启鉴权
    #[serde(deserialize_with = "crate::server::common::deserialize_bool_from_int")]
    pub is_auth: Option<bool>,
    /// 鉴权白名单
    pub auth_white_list: Option<Vec<String>>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum RouteStatus {
    /// 停用
    #[default]
    Disable = 0,
    /// 启用
    Ok = 1,
}

crud!(Route {});
htmlsql_select_page!(list_page(param: &RouteListReq) -> Route => "src/server/db/mapper/route.html");
