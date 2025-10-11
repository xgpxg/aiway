use crate::server::db::models::service::{Service, ServiceStatus};
use protocol::common::req::PageReq;
use protocol::gateway::service::LbStrategy;
use protocol::impl_pagination;
use rocket::serde::{Deserialize, Serialize};

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
    pub page: PageReq,
    /// 模糊搜索：服务名/描述
    pub filter_text: Option<String>,
    /// 状态
    pub status: Option<ServiceStatus>,
}
impl_pagination!(ServiceListReq);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatusReq {
    pub id: i64,
    pub status: ServiceStatus,
}
