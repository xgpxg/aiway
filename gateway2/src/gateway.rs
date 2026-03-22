use crate::Args;
use crate::context::RequestContext;
use crate::handler::plugin;
use crate::handler::plugin::PluginType;
use crate::model_proxy::ModelFactory;
use aiway_plugin_v2::{ResponseHeader, async_trait};
use aiway_protocol::common::header::Headers;
use aiway_protocol::context::{HttpContext, RequestHeader, SessionExt};
use bytes::Bytes;
use http::Uri;
use ip2region::IpValueExt;
use pingora::{Error, ErrorType};
use pingora_core::prelude::HttpPeer;
use pingora_core::protocols::http::subrequest::server::HttpSession;
use pingora_proxy::subrequest::Ctx;
use pingora_proxy::{FailToProxy, ProxyHttp, Session};
use serde_json::json;
use std::time::Duration;
use tokio_stream::StreamExt;

pub struct Gateway {}

impl Gateway {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ProxyHttp for Gateway {
    type CTX = HttpContext;

    fn new_ctx(&self) -> Self::CTX {
        HttpContext::default()
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>, Box<Error>> {
        let backend_addr = ctx
            .get_routing_url()
            .ok_or_else(|| Error::new_str("Routing URL not found in context"))?;

        // 转发到本地处理
        if backend_addr.ends_with(".sock") {
            return Ok(Box::new(HttpPeer::new_uds(
                backend_addr,
                false,
                "".to_string(),
            )?));
        }

        let uri: Uri = backend_addr
            .parse()
            .map_err(|_| Error::new_str("Failed to parse backend URI"))?;

        let host = uri
            .host()
            .ok_or_else(|| Error::new_str("Backend URI missing host"))?;

        let port = uri.port_u16().unwrap_or_else(|| {
            if uri.scheme_str() == Some("https") {
                443
            } else {
                80
            }
        });

        let peer = Box::new(HttpPeer::new((host, port), false, host.to_string()));

        log::debug!("Proxying to {}:{} with SNI {}", host, port, host);

        Ok(peer)
    }

    /// 在向后端发送请求之前
    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<bool, Box<Error>> {
        // 路由匹配
        if let Err(e) = crate::handler::routing_handle(session, ctx).await {
            session.respond_error_with_body(e.0, e.1.into()).await?;
            return Ok(true);
        }

        // 鉴权
        if let Err(e) = crate::handler::auth_handle(session, ctx).await {
            session.respond_error_with_body(e.0, e.1.into()).await?;
            return Ok(true);
        }

        // 执行路由请求阶段插件，可在此处修改http头部
        if let Err(e) =
            plugin::run_on_request(PluginType::Route, session.req_header_mut(), ctx).await
        {
            session.respond_error_with_body(e.0, e.1.into()).await?;
            return Ok(true);
        }

        // 负载均衡，查找服务实例
        if let Err(e) = crate::handler::lb_handle(session, ctx).await {
            session.respond_error_with_body(e.0, e.1.into()).await?;
            return Ok(true);
        }

        //session.set_keepalive(Some(60));

        Ok(false)
    }

    async fn early_request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<()>
    where
        Self::CTX: Send + Sync,
    {
        // 预处理
        if let Err(e) = crate::handler::pre_handle(session, ctx).await {
            return session.respond_error_with_body(e.0, e.1.into()).await;
        }

        // 前置安全校验
        if let Err(e) = crate::handler::security_check(session, ctx).await {
            return session.respond_error_with_body(e.0, e.1.into()).await;
        }

        // 执行全局请求阶段插件
        if let Err(e) =
            plugin::run_on_request(PluginType::Global, session.req_header_mut(), ctx).await
        {
            return session.respond_error_with_body(e.0, e.1.into()).await;
        }

        Ok(())
    }

    /// 在向后端发送请求之前执行
    ///
    /// 这里修改Header将不生效
    async fn request_body_filter(
        &self,
        _session: &mut Session,
        body: &mut Option<Bytes>,
        _end_of_stream: bool,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<()>
    where
        Self::CTX: Send + Sync,
    {
        // 执行插件
        if let Err(e) = plugin::run_on_request_body(PluginType::Route, body, ctx).await {
            log::error!("Request handle error: {}", e);
            return Ok(());
        }

        Ok(())
    }

    async fn upstream_response_filter(
        &self,
        session: &mut Session,
        resp: &mut ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<(), Box<Error>> {
        // 执行路由响应阶段插件
        if let Err(e) = plugin::run_on_response(PluginType::Route, resp, ctx).await {
            log::error!("Request handle error: {}", e);
            return Ok(());
        }

        // 执行路由响应阶段插件
        if let Err(e) = plugin::run_on_response(PluginType::Global, resp, ctx).await {
            log::error!("Request handle error: {}", e);
            return Ok(());
        }

        // 响应处理
        if let Err(e) = crate::handler::response_handle(session, resp, ctx).await {
            log::error!("Response handle error: {}", e);
            return Ok(());
        }

        Ok(())
    }

    fn upstream_response_body_filter(
        &self,
        _: &mut Session,
        body: &mut Option<Bytes>,
        _: bool,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<Option<Duration>> {
        // 执行路由响应体阶段插件，可在此处修改响应body
        if let Err(e) = plugin::run_on_response_body(PluginType::Route, body, ctx) {
            log::error!("Request handle error: {}", e);
        }
        // 执行全局响应体阶段插件，可在此处修改响应body
        if let Err(e) = plugin::run_on_response_body(PluginType::Global, body, ctx) {
            log::error!("Request handle error: {}", e);
        }
        Ok(None)
    }

    async fn logging(&self, session: &mut Session, _e: Option<&Error>, ctx: &mut Self::CTX)
    where
        Self::CTX: Send + Sync,
    {
        // 日志记录
        //crate::handler::log_handle(session, resp, ctx).await;
        // 清理
        crate::handler::cleanup_handle(session, ctx).await;
    }
    async fn fail_to_proxy(
        &self,
        session: &mut Session,
        e: &Error,
        ctx: &mut Self::CTX,
    ) -> FailToProxy
    where
        Self::CTX: Send + Sync,
    {
        log::error!("Fail to proxy: {}", e);

        FailToProxy {
            error_code: match e.etype {
                ErrorType::CustomCode(_, code) => code,
                _ => 502,
            },
            can_reuse_downstream: true,
        }
    }
}
