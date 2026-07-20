//! # 模型调用日志
//! 记录每次模型调用的 token 用量和性能指标。

use serde::{Deserialize, Serialize};

/// 模型调用日志
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCallLog {
    /// 请求 ID
    pub request_id: String,
    /// 模型名称
    pub model_name: String,
    /// 提供商名称
    pub provider_name: String,
    /// 请求时间（毫秒时间戳）
    pub request_time: i64,
    /// 响应时间（毫秒时间戳）
    pub response_time: i64,
    /// 总耗时（毫秒）
    pub elapsed: i64,
    /// 首 Token 响应时间（毫秒），仅流式请求有效
    pub ttft_ms: Option<i64>,
    /// 网关返回给客户端的 HTTP 状态码
    pub status_code: u16,
    /// 是否 SSE 流式响应
    pub is_stream: bool,
    /// prompt token 数量
    pub prompt_tokens: Option<i64>,
    /// completion token 数量
    pub completion_tokens: Option<i64>,
    /// 总 token 数量
    pub total_tokens: Option<i64>,
    /// 处理请求的节点地址
    pub node_address: String,
}
