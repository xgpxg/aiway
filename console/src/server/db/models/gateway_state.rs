use derive_builder::Builder;
use rbatis::crud;
use rbatis::rbdc::DateTime;
use rocket::serde::{Deserialize, Serialize};

/// 网关节点信息
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct GatewayState {
    pub id: i64,
    /// 网关节点个数
    /// 取值：gateway_node表的所有节点数量
    pub node_count: usize,
    /// 运行中节点个数
    /// 取值：gateway_node表的运行中节点数量
    pub running_node_count: usize,
    /// 宕机节点个数
    /// 取值：gateway_node表的宕机节点数量
    pub down_node_count: usize,
    /// 当前连接总数
    /// 取值：gateway_node表的所有节点的连接总数
    pub total_connection_count: usize,
    /// 今日请求数
    pub today_request_count: usize,
    /// 总请求数
    pub total_request_count: usize,
    /// 平均QPS（上一统计周期内）
    pub avg_qps: f64,
    /// 平均响应时间（上一统计周期内）
    pub avg_response_time: f64,
    /// 更新时间
    #[serde(serialize_with = "crate::server::common::serialize_datetime")]
    pub update_time: Option<DateTime>,
}

crud!(GatewayState {});
