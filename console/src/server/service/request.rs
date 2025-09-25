use crate::server::db::models::service::Service;
use protocol::gateway::service::LbStrategy;
use rocket::serde::{Deserialize, Serialize};
use protocol::common::req::PageReq;
use protocol::impl_pagination;
use crate::impl_rb_page;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAddOrUpdateReq {
    pub id: Option<i64>,
    /// 服务名称，全局唯一
    pub name: String,
    /// 描述
    pub description: String,
    /// 服务节点
    #[serde(default = "Vec::default")]
    pub nodes: Vec<String>,
    /// 负载均衡策略，可选值：random | round_robin
    #[serde(default = "LbStrategy::default")]
    pub lb: LbStrategy,
}
impl From<ServiceAddOrUpdateReq> for Service {
    fn from(service: ServiceAddOrUpdateReq) -> Self {
        Service {
            id: service.id,
            name: service.name.into(),
            description: service.description.into(),
            status: Some(Default::default()),
            nodes: service.nodes.into(),
            lb: service.lb.into(),
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
pub struct ServiceListReq {
    /// 模糊搜索：服务名/描述
    pub filter_text: Option<String>,
    pub page: PageReq,
}
impl_pagination!(ServiceListReq);
impl_rb_page!(ServiceListReq);
