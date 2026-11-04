use crate::server::service::ServiceListReq;
use derive_builder::Builder;
use aiway_protocol::gateway::service::{LbStrategy, ServiceNode};
use rbatis::rbdc::DateTime;
use rbatis::{crud, htmlsql_select_page};
use rocket::serde::{Deserialize, Serialize};

/// 路由配置
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct Service {
    pub id: Option<i64>,
    /// 服务名称，全局唯一
    pub name: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 状态，0停用 1启用
    pub status: Option<ServiceStatus>,
    /// 服务节点，JSON数组，支持两种形态（兼容老数据）：
    /// 1. 纯地址字符串 `["http://127.0.0.1:8080"]`，权重默认为 1
    /// 2. 带权重的对象 `[{"addr":"10.0.0.1:8080","weight":3}]`，供加权类负载均衡策略使用
    /// 这里仅保存节点地址与权重即可，原因是：
    /// 1. 网关仅需要知道节点地址即可发起调用
    /// 2. console可能与节点之间的网络不通，因此不能确保能够在console检测节点状态，进而没必要保存节点状态
    /// 3. 考虑在网关处或者其他方式实现节点故障处理
    pub nodes: Option<Vec<ServiceNode>>,
    /// 负载均衡策略，可选值：random | round_robin | weighted_round_robin | weighted_random | least_conn | ip_hash
    pub lb: Option<LbStrategy>,
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
pub enum ServiceStatus {
    /// 停用
    #[default]
    Disable,
    /// 启用
    Ok,
}

crud!(Service {});
htmlsql_select_page!(list_page(param: &ServiceListReq) -> Service => "src/server/db/mapper/service.html");
