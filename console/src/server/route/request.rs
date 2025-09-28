use crate::server::db::models::route::Route;
use protocol::common::req::PageReq;
use protocol::gateway::plugin::ConfiguredPlugin;
use protocol::impl_pagination;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteAddOrUpdateReq {
    pub id: Option<i64>,
    /// 路由名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 域名匹配
    pub host: Option<String>,
    /// 前缀
    pub prefix: Option<String>,
    /// 是否移除前缀
    #[serde(default = "Default::default")]
    pub strip_prefix: bool,
    /// 路径匹配
    pub path: String,
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
}

impl From<RouteAddOrUpdateReq> for Route {
    fn from(req: RouteAddOrUpdateReq) -> Self {
        Route {
            id: req.id.into(),
            name: req.name.into(),
            description: req.description,
            status: Some(Default::default()),
            host: req.host,
            prefix: req.prefix,
            strip_prefix: (req.strip_prefix as i8).into(),
            path: req.path.into(),
            service: req.service.into(),
            header: req.header.into(),
            query: req.query.into(),
            pre_filters: req.pre_filters.into(),
            post_filters: req.post_filters.into(),
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
    /// 模糊搜索：路由名称、描述、域名、前缀、路径
    pub filter_text: Option<String>,
    pub page: PageReq,
}
impl_pagination!(RouteListReq);
