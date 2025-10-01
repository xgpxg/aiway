use derive_builder::Builder;
use rbatis::crud;
use rbatis::rbdc::DateTime;
use rocket::serde::{Deserialize, Serialize};

/// 网关节点信息
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct GatewayNodeStateLog {
    pub id: Option<i64>,
    /// 节点ID
    pub node_id: Option<String>,
    /// 状态产生时的时间戳，单位毫秒
    pub ts: Option<String>,
    ///  操作系统信息
    pub os: Option<String>,
    /// 节点名称
    pub host_name: Option<u16>,
    /// CPU使用率
    pub cpu_usage: Option<f32>,
    /// 内存总大小
    pub mem_total: Option<usize>,
    /// 可用内存
    pub mem_free: Option<usize>,
    /// 已用内存
    pub mem_used: Option<usize>,
    /// 磁盘总容量
    pub disk_total: Option<usize>,
    /// 可用磁盘容量
    pub disk_free: Option<usize>,
    /// 网络接收字节数
    pub net_rx: Option<usize>,
    /// 网络发送字节数
    pub net_tx: Option<usize>,
    /// TCP连接数
    pub net_tcp_conn_count: Option<usize>,
    /// 累计请求次数
    pub request_count: Option<usize>,
    /// 累计无效请求次数
    pub request_invalid_count: Option<usize>,
    /// 累计响应成功次数
    pub response_2xx_count: Option<usize>,
    /// 累计3xx响应次数
    pub response_3xx_count: Option<usize>,
    /// 累计4xx响应次数
    pub response_4xx_count: Option<usize>,
    /// 累计5xx响应次数
    pub response_5xx_count: Option<usize>,
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

crud!(GatewayNodeStateLog {});
