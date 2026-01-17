use crate::server::db::models::route::{Route, RouteStatus};
use busi::req::PageReq;
use aiway_protocol::gateway::GlobalFilter;
use aiway_protocol::gateway::plugin::ConfiguredPlugin;
use busi::impl_pagination;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteAddOrUpdateReq {
    pub id: Option<i64>,
    /// 路由名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// host匹配，默认 *
    #[serde(default = "default_host")]
    pub host: String,
    /// 路径匹配
    pub path: String,
    #[serde(default = "Default::default")]
    pub methods: Vec<String>,
    /// 目标服务
    pub service: String,
    /// header匹配
    #[serde(default = "Default::default")]
    pub header: BTreeMap<String, String>,
    /// query匹配
    #[serde(default = "Default::default")]
    pub query: BTreeMap<String, String>,
    /// 前置过滤器
    #[serde(default = "Default::default")]
    pub pre_filters: Vec<ConfiguredPlugin>,
    /// 后置过滤器
    #[serde(default = "Default::default")]
    pub post_filters: Vec<ConfiguredPlugin>,
    /// 是否需要认证
    pub is_auth: Option<bool>,
    /// 认证白名单
    pub auth_white_list: Option<Vec<String>>,
}

fn default_host() -> String {
    "*".into()
}

impl From<RouteAddOrUpdateReq> for Route {
    fn from(req: RouteAddOrUpdateReq) -> Self {
        Route {
            id: req.id,
            name: req.name.into(),
            description: req.description,
            status: Some(Default::default()),
            host: req.host.into(),
            path: req.path.into(),
            service: req.service.into(),
            methods: req.methods.into(),
            header: req.header.into(),
            query: req.query.into(),
            pre_filters: req.pre_filters.into(),
            post_filters: req.post_filters.into(),
            is_auth: req.is_auth,
            auth_white_list: req.auth_white_list,
            create_user_id: None,
            update_user_id: None,
            create_time: None,
            update_time: None,
            remark: None,
            is_delete: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteListReq {
    pub page: PageReq,
    /// 模糊搜索：路由名称、描述、域名、前缀、路径
    pub filter_text: Option<String>,
    /// 关联服务
    pub service: Option<String>,
}
impl_pagination!(RouteListReq);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatusReq {
    pub id: i64,
    pub status: RouteStatus,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGlobalFilterConfigReq {
    #[serde(flatten)]
    pub inner: GlobalFilter,
}
