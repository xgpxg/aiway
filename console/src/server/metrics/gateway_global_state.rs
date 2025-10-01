use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayGlobalState {
    // 网关节点个数
    // 取值：gateway_node表的节点数量
    pub node_count: usize,
    // 运行中节点个数
    // 取值：gateway_node表的运行中节点数量
    pub running_node_count: usize,
    // 宕机节点个数
    pub down_node_count: usize,
    // 当前连接总数
    pub total_connection_count: usize,
    // 今日请求数
    pub today_request_count: usize,
    // 总请求数
    pub total_request_count: usize,
    // 平均QPS（上一统计周期内）
    pub avg_qps: f64,
    // 平均响应时间（上一统计周期内）
    pub avg_response_time: f64,
}
