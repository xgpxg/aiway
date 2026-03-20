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
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>, Box<Error>> {
        log::info!("new_ctx");
        let path = session.get_path();
        match path {
            p if p.starts_with("/v1/model/") => {
                let provider = ctx.inner_state.get_model_provider().unwrap();
                let api_url = &provider.api_url;

                // 解析 URL 并提取 host 和 port
                let uri: Uri = api_url
                    .parse()
                    .map_err(|_| Error::new_str("Failed to parse API URL"))?;

                let host = uri
                    .host()
                    .ok_or_else(|| Error::new_str("API URL missing host"))?;

                let port = uri.port_u16().unwrap_or_else(|| {
                    if uri.scheme_str() == Some("https") {
                        443
                    } else {
                        80
                    }
                });
                log::info!(
                    "Creating upstream peer: host={}, port={}, scheme={:?}",
                    host,
                    port,
                    uri.scheme_str()
                );
                session.set_path("/v1/chat/completions");
                session.set_request_header("Host", host);

                if let Some(api_key) = provider.api_key {
                    session.set_request_header("Authorization", &format!("Bearer {}", api_key));
                    log::info!("Authorization: {}", api_key);
                }

                let peer = HttpPeer::new((host, port), true, host.to_string());

                Ok(Box::new(peer))
            }
            _ => {
                unreachable!()
            }
        }
    }

    /// 在向后端发送请求之前
    async fn request_filter(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<bool, Box<Error>> {
        let path = session.get_path();
        match path {
            p if p.starts_with("/v1/mcp/") => {
                let body = session.read_request_body().await?.unwrap_or_default();
                let response = mcp_post_endpoint(&p, body).await.unwrap();
                forward_reqwest_to_pingora(response, session).await?;
            }
            p if p.starts_with("/v1/model/") => {
                let body = session.read_request_body().await?.unwrap_or_default();
                let response = match model_endpoint(&p, body).await {
                    Ok(response) => response,
                    Err(e) => {
                        // TODO 修改错误类型
                        return session.respond_error(502).await.map(|_| true);
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
