use crate::server::db::models::service::ServiceStatus;
use aiway_protocol::common::req::PageReq;
use aiway_protocol::gateway::service::LbStrategy;
use aiway_protocol::impl_pagination;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAddReq {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceUpdateReq {
    pub id: i64,
    /// 描述
    pub description: Option<String>,
    /// 服务节点
    pub nodes: Option<Vec<String>>,
    /// 负载均衡策略，可选值：random | round_robin
    pub lb: Option<LbStrategy>,
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
