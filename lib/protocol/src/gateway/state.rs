use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    pub timestamp: i64,
    /// 系统状态
    pub system_state: SystemState,
    /// 计数器
    pub counter: Counter,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemState {
    /// 操作系统及版本，如: Ubuntu 22.04
    pub os: String,
    /// 主机名
    pub host_name: String,
    /// cpu 使用率
    pub cpu_usage: f32,
    /// 内存状态
    pub mem_state: MemState,
    /// 磁盘状态
    pub disk_state: DiskState,
    /// 网络状态
    pub net_state: NetState,
}

/// 内存状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemState {
    /// 总内存，单位：Bytes
    pub total: u64,
    /// 空闲内存，单位：Bytes
    pub free: u64,
    /// 使用内存，单位：Bytes
    pub used: u64,
}

/// 磁盘状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskState {
    /// 总空间，单位：Bytes
    pub total: u64,
    /// 空闲空间，单位：Bytes
    pub free: u64,
}

/// 网络状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetState {
    /// 接收的字节数
    pub rx: u64,
    /// 发送的字节数
    pub tx: u64,
    /// TCP连接数
    pub tcp_conn_count: usize,
}

/// 计数器
///
/// 注意：该计数器仅统计自上次上报到现在为止这段时间内的数据。
/// 上报到控制台后，由控制台汇总。
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Counter {
    /// 自从上次统计到现在的请求数
    /// 统计范围：
    /// - 所有进入到网关的请求
    pub request_count: usize,
    /// 非法请求数
    /// 统计范围：
    /// - 由安全组件拦截到的无效、非法、恶意请求等
    /// 拦截的请求会返回403错误，也会被计算到4xx响应数中
    pub request_invalid_count: usize,
    /// 自从上次统计到现在的 2xx 响应数
    pub response_2xx_count: usize,
    /// 自从上次统计到现在的 3xx 响应数
    pub response_3xx_count: usize,
    /// 自从上次统计到现在的 4xx 响应数
    /// 统计范围：
    /// - 前置安全验证不通过的
    /// - API Key验证失败的
    /// - 客户端错误（一般是参数错误）
    pub response_4xx_count: usize,
    /// 自从上次统计到现在的 5xx 响应数。
    /// 5xx为重要关注指标
    pub response_5xx_count: usize,
    /// 自从上次统计到现在的请求累计响应时间，单位：毫秒
    /// 统计范围：
    /// - 每个请求的响应时间
    ///
    /// 统计周期内的平均响应时间 = response_time_since_last / request_count
    pub response_time_since_last: usize,
}

impl State {
    pub fn reset_counter(&mut self) {
        self.counter.request_count = 0;
        self.counter.response_time_since_last = 0;
        self.counter.request_invalid_count = 0;
        self.counter.response_2xx_count = 0;
        self.counter.response_3xx_count = 0;
        self.counter.response_4xx_count = 0;
        self.counter.response_5xx_count = 0;
    }
}
