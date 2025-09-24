pub struct State {
    pub os: String,
    pub cpu_usage: f64,
    pub mem_state: MemState,
    pub disk_state: DiskState,
    pub net_state: NetState,
    pub counter: Counter,
}

/// 内存状态
struct MemState {
    total: u64,
    free: u64,
    used: u64,
}

/// 磁盘状态
struct DiskState {
    total: u64,
    free: u64,
}

/// 网络状态
struct NetState {
    /// 接收的字节数
    rx: u64,
    /// 发送的字节数
    tx: u64,
}

/// 计数器
struct Counter {
    /// 总请求数
    request_count: u64,
    /// 响应时间
    response_time: f64,
    /// 错误数
    error_count: u64,
}

impl State {}
