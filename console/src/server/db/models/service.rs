use crate::server::service::ServiceListReq;
use derive_builder::Builder;
use protocol::gateway::service::LbStrategy;
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
    /// 服务节点，JSON数组，支持IP和域名，如["http://127.0.0.1:8080"]
    pub nodes: Option<Vec<String>>,
    /// 负载均衡策略，可选值：random | round_robin
    pub lb: Option<LbStrategy>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ServiceStatus {
    #[default]
    Disable = 0,
    Ok = 1,
}

crud!(Service {});
htmlsql_select_page!(list_page(param: &ServiceListReq) -> Service => "src/server/db/mapper/service.html");
