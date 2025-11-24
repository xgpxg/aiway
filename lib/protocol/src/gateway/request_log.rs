use serde::{Deserialize, Serialize};

/// 网关请求日志
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestLog {
    /// 请求ID，由网关统一生成，唯一的请求标识，标准的36位UUID
    pub request_id: String,
    /// 客户端IP地址，通过该地址定位到客户端所在区域
    pub client_ip: String,
    /// 客户端所在国家
    pub client_country: Option<String>,
    /// 客户端所在省份
    pub client_province: Option<String>,
    /// 客户端所在城市
    pub client_city: Option<String>,
    /// 请求方式，如GET、POST、PUT、DELETE等
    pub method: String,
    /// 请求路径，不含query参数
    pub path: String,
    /// 请求时间戳，即网关收到请求的时刻，单位：毫秒
    pub request_time: i64,
    /// 响应时间戳，即网关将响应发送到客户端前的时刻，单位：毫秒
    pub response_time: i64,
    /// 耗时。统计从接收到请求开始到响应客户端之前的时间，单位：毫秒。
    pub elapsed: i64,
    /// HTTP状态码
    pub status_code: u16,
    /// 响应大小，流式响应为None
    pub response_size: Option<usize>,
    /// 取Header里的User-Agent
    pub user_agent: Option<String>,
    /// 取Header里的Referer
    pub referer: Option<String>,
    /// 网关节点地址，格式：ip:port，该字段用于记录请求被哪个网关节点处理
    pub node_address: String,
}
