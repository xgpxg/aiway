use crate::server::db::models::gateway_node::GatewayNode;
use crate::server::db::models::gateway_node_state::GatewayNodeState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayNodeListRes {
    #[serde(flatten)]
    pub inner: GatewayNode,
    pub state: Option<GatewayNodeState>,
}

/// 用量信息，用于图表展示。
/// - CPU用量
/// - 内存用量
/// - 网络用量
/// - 连接数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRes {
    /// 毫秒时间戳
    pub t: i64,
    /// 数值
    pub v: f32,
}
