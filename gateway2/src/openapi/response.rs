//! # 响应处理
//!
use reqwest::Response;
use aiway_protocol::context::ResponseContext;

pub struct GatewayResponse;

impl GatewayResponse {
    pub async fn from_response(response: Response, _ctx: &ResponseContext) -> Self {
        // TODO: 实现响应转换
        Self
    }
}
