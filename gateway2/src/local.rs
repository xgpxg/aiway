use crate::Args;
use crate::mcp_proxy::mcp_post_endpoint;
use crate::model_proxy::{ModelFactory, model_endpoint};
use aiway_plugin_v2::{ResponseHeader, async_trait};
use aiway_protocol::context::{HttpContext, RequestHeader, SessionExt};
use bytes::Bytes;
use http::Uri;
use pingora::Error;
use pingora_core::prelude::HttpPeer;
use pingora_proxy::{ProxyHttp, Session};
use tokio_stream::StreamExt;
use aiway_protocol::common::constants::{MCP_API_PREFIX, MODEL_API_PREFIX};

pub struct Local {
    args: Args,
}

impl Local {
    pub fn new(args: &Args) -> Self {
        Self { args: args.clone() }
    }
}

#[async_trait]
impl ProxyHttp for Local {
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
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<bool, Box<Error>> {
        let path = session.get_path();
        match path {
            // 处理MCP
            p if p.starts_with(MCP_API_PREFIX) => {
                let body = session.read_request_body().await?.unwrap_or_default();
                let response = mcp_post_endpoint(&p, body).await.unwrap();
                forward_reqwest_to_pingora(response, session).await?;
            }
            // 处理模型
            p if p.starts_with(MODEL_API_PREFIX) => {
                let body = session.read_request_body().await?.unwrap_or_default();
                let response = match model_endpoint(&p, body).await {
                    Ok(response) => response,
                    Err(e) => {
                        let (status, message) = e.into_status_message();
                        return session
                            .respond_error_with_body(
                                status,
                                Bytes::copy_from_slice(message.as_bytes()),
                            )
                            .await
                            .map(|_| true);
                    }
                };
                forward_reqwest_to_pingora(response, session).await?;
            }
            _ => {}
        }
        Ok(true)
    }
}

async fn forward_reqwest_to_pingora(
    mut response: reqwest::Response,
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

    // 转发body
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
