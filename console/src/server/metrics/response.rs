use rocket::serde::{Deserialize, Serialize};

/// 网关节点信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayState {
    pub id: i64,
    /// 网关节点个数
    /// 取值：gateway_node表的所有节点数量
    pub node_count: usize,
    /// 运行中节点个数
    /// 取值：gateway_node表的运行中节点数量
    pub online_node_count: usize,
    /// 宕机节点个数
    /// 取值：gateway_node表的宕机节点数量
    pub offline_node_count: usize,
    /// 平均QPS
    /// 取值：sum(运行中的节点的avg_qps)
    pub avg_qps: usize,
    /// 平均响应时间
    /// 取值：sum(运行中的节点的avg_response_time) / 节点个数 取整
    pub avg_response_time: usize,
    /// 今日请求数
    pub request_today_count: usize,
    /// 累计请求次数
    pub request_count: usize,
    /// 累计无效请求次数
    pub request_invalid_count: usize,
    /// 累计响应成功次数
    pub response_2xx_count: usize,
    /// 累计3xx响应次数
    pub response_3xx_count: usize,
    /// 累计4xx响应次数
    pub response_4xx_count: usize,
    /// 累计5xx响应次数
    pub response_5xx_count: usize,
    /// 当前连接总数
    /// 取值：gateway_node表的所有节点的连接总数
    pub http_connect_count: usize,
    /// 未读的错误信息数量
    pub error_count: usize,
    /// 未读的警告信息数量
    pub warn_count: usize,
    /// 未读的提示信息数量
    pub info_count: usize,
}
