use derive_builder::Builder;
use rbatis::crud;
use rbatis::rbdc::DateTime;
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
    /// HTTP连接数
    pub http_connect_count: isize,
    /// 平均QPS(统计周期内)
    pub avg_qps: usize,
    /// 平均响应时间，单位：毫秒
    pub avg_response_time: usize,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

crud!(GatewayNodeState {});
