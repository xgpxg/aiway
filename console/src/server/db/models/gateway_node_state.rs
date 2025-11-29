use derive_builder::Builder;
use rbatis::executor::Executor;
use rbatis::rbdc::DateTime;
use rbatis::{crud, htmlsql};
use rocket::serde::{Deserialize, Serialize};

/// 网关节点状态记录
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct GatewayNodeState {
    pub id: Option<i64>,
    /// 节点ID
    pub node_id: String,
    /// 状态产生时的时间戳，单位毫秒
    pub ts: i64,
    ///  操作系统信息
    pub os: Option<String>,
    /// 节点名称
    pub host_name: Option<u16>,
    /// CPU使用率
    pub cpu_usage: f32,
    /// 内存总大小
    pub mem_total: u64,
    /// 可用内存
    pub mem_free: u64,
    /// 已用内存
    pub mem_used: u64,
    /// 磁盘总容量
    pub disk_total: u64,
    /// 可用磁盘容量
    pub disk_free: u64,
    /// 网络接收字节数
    pub net_rx: u64,
    /// 网络发送字节数
    pub net_tx: u64,
    /// TCP连接数
    pub net_tcp_conn_count: usize,
    /// HTTP连接数
    pub http_connect_count: isize,
    /// SSE连接数
    pub sse_connect_count: isize,
    /// 平均QPS(统计周期内)
    pub avg_qps: usize,
    //////////////////////////// 区间内统计 ////////////////////////////
    /// 区间请求次数
    pub interval_request_count: usize,
    /// 区间无效请求次数
    pub interval_request_invalid_count: usize,
    /// 区间响应成功次数
    pub interval_response_2xx_count: usize,
    /// 区间3xx响应次数
    pub interval_response_3xx_count: usize,
    /// 区间4xx响应次数
    pub interval_response_4xx_count: usize,
    /// 区间5xx响应次数
    pub interval_response_5xx_count: usize,
    /// 区间平均响应时间，单位：毫秒
    pub interval_avg_response_time: usize,
    //////////////////////////// 累计统计 ////////////////////////////
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
    /// 累计平均响应时间，单位：毫秒
    pub avg_response_time: usize,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

crud!(GatewayNodeState {});

// 查询网关节点的最新状态
htmlsql!(lastest_state(rb: &dyn Executor, node_ids :&Vec<String>)  -> Vec<GatewayNodeState> => "src/server/db/mapper/gateway_node_state.html");
