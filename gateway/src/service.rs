use crate::mcp_proxy::{handle_mcp_request};
use crate::model_proxy::handle_model_request;
use plugin_manager::async_trait;
use aiway_protocol::common::constants::{MCP_API_PREFIX, MODEL_API_PREFIX};
use aiway_protocol::context::{HttpContext, RequestExt};
use bytes::Bytes;
use pingora::Error;
use pingora::http::ResponseHeader;
use pingora::prelude::{HttpPeer, ProxyHttp, Session};
use tokio_stream::StreamExt;

/// 本地服务
///
/// 用于处理网关自身业务请求和响应，如模型代理、MCP代理等等。
pub struct LocalService {}

impl LocalService {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ProxyHttp for LocalService {
    type CTX = HttpContext;

    fn new_ctx(&self) -> Self::CTX {
        HttpContext::default()
    }

    async fn upstream_peer(
        &self,
        _: &mut Session,
        _: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>, Box<Error>> {
        unreachable!()
    }

    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<bool, Box<Error>> {
        let path = session.req_header().get_path();

        // MCP API 处理
        if path.starts_with(MCP_API_PREFIX) {
            return handle_mcp_request(session, path.as_str()).await;
        }

        // 模型 API 处理
        if path.starts_with(MODEL_API_PREFIX) {
            return handle_model_request(session, path.as_str(), ctx).await;
        }

        Ok(true)
    }
}

/// 构建 Pingora 响应头
pub fn build_response_header(response: &reqwest::Response) -> ResponseHeader {
    let status = response.status();
    let mut pingora_header = ResponseHeader::build(status, None).unwrap();

    for (name, value) in response.headers().iter() {
        pingora_header
            .insert_header(name.to_string(), value.as_bytes().to_vec())
            .unwrap();
    }
    pingora_header
}

/// 发送错误响应（字符串消息）
pub async fn send_error_response(
    session: &mut Session,
    status: u16,
    message: String,
) -> pingora::Result<()> {
    session
        .respond_error_with_body(status, Bytes::copy_from_slice(message.as_bytes()))
        .await
}

// /// 发送错误响应（字节数组）
// pub async fn send_error_response_with_bytes(
//     session: &mut Session,
//     status: u16,
//     message: &[u8],
// ) -> pingora::Result<()> {
//     session
//         .respond_error_with_body(status, Bytes::copy_from_slice(message))
//         .await
// }

/// 转发 reqwest 响应到 Pingora
#[allow(dead_code)]
pub(crate) async fn forward_reqwest_to_pingora(
    response: reqwest::Response,
    session: &mut Session,
) -> pingora::Result<bool> {
    // 构建 Pingora 响应头
    let status = response.status();
    let mut pingora_header = ResponseHeader::build(status, None)?;

    // 复制 Header
    for (name, value) in response.headers().iter() {
        pingora_header.insert_header(name.to_string(), value.as_bytes().to_vec())?;
    }

    // 发送响应头
    session
        .write_response_header(Box::new(pingora_header), false)
        .await?;

    // 转发 body
    let mut body_stream = response.bytes_stream();
    while let Some(chunk_result) = body_stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                session.write_response_body(Some(chunk), false).await?;
            }
            Err(e) => {
                log::error!("Error reading body: {}", e);
                return session.respond_error(502).await.map(|_| true);
            }
        }
    }

    // 发送结束标记
    session.write_response_body(None, true).await?;

    Ok(true)
}
