use crate::Args;
use crate::handler::plugin;
use crate::handler::plugin::PluginType;
use aiway_plugin_v2::async_trait;
use aiway_protocol::common::header::Headers;
use aiway_protocol::context::parts::SerdeParts;
use aiway_protocol::context::{HttpContext, RequestExt};
use bytes::Bytes;
use http::Uri;
use pingora::Error;
use pingora::http::{ ResponseHeader};
use pingora_core::prelude::HttpPeer;
use pingora_core::protocols::http::ServerSession;
use pingora_proxy::{ProxyHttp, Session};
use std::ops::{Deref, DerefMut};
use std::time::Duration;

pub struct Gateway {
    args: Args,
}

impl Gateway {
    pub fn new(args: &Args) -> Self {
        Self { args: args.clone() }
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
        session: &mut Session,
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

        let tls = uri
            .scheme_str()
            .map(|s| s == "https" || s == "wss")
            .unwrap_or(false);

        let port = uri.port_u16().unwrap_or_else(|| if tls { 443 } else { 80 });

        let peer = Box::new(HttpPeer::new((host, port), tls, host.to_string()));

        let header_mut = session.req_header_mut();
        header_mut.set_request_header("Host", host);
        header_mut.remove_header(Headers::AUTHORIZATION);

        log::debug!("Proxying to {}:{} with SNI {}", host, port, host);

        Ok(peer)
    }

    /// 在向后端发送请求之前
    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<bool, Box<Error>> {
        // 记录原始Parts
        ctx.insert_state(
            HttpContext::REQUEST_RAW_PARTS,
            SerdeParts::from(session.req_header().deref()),
        );

        // 执行全局请求阶段插件
        if let Err(e) =
            plugin::run_on_request(PluginType::Global, session.req_header_mut(), ctx).await
        {
            ctx.insert_state(HttpContext::RESPONSE_SERDE_PARTS, from_status_code(e.0));
            session.respond_error_with_body(e.0, e.1.into()).await?;
            return Ok(true);
        }

        // 路由匹配
        if let Err(e) = crate::handler::routing_handle(session, ctx).await {
            ctx.insert_state(HttpContext::RESPONSE_SERDE_PARTS, from_status_code(e.0));
            session.respond_error_with_body(e.0, e.1.into()).await?;
            return Ok(true);
        }

        // 鉴权
        if let Err(e) = crate::handler::auth_handle(session, ctx).await {
            ctx.insert_state(HttpContext::RESPONSE_SERDE_PARTS, from_status_code(e.0));

            session.respond_error_with_body(e.0, e.1.into()).await?;
            return Ok(true);
        }

        // 执行路由请求阶段插件，可在此处修改http头部
        if let Err(e) =
            plugin::run_on_request(PluginType::Route, session.req_header_mut(), ctx).await
        {
            ctx.insert_state(HttpContext::RESPONSE_SERDE_PARTS, from_status_code(e.0));

            session.respond_error_with_body(e.0, e.1.into()).await?;
            return Ok(true);
        }

        // 负载均衡，查找服务实例
        if let Err(e) = crate::handler::lb_handle(session, ctx).await {
            ctx.insert_state(HttpContext::RESPONSE_SERDE_PARTS, from_status_code(e.0));

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
        // 在这里进行连接计数等，不修改任何客户端原始数据
        if let Err(e) = crate::handler::pre_handle(session, ctx).await {
            return session.respond_error_with_body(e.0, e.1.into()).await;
        }

        // 防护墙安全校验
        // 被防护墙拦截的请求不会记录日志
        if let Err(e) = crate::handler::firewall_check(session, ctx).await {
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
            // TODO 错误处理
            return Ok(());
        }

        // 执行全局响应阶段插件
        if let Err(e) = plugin::run_on_response(PluginType::Global, resp, ctx).await {
            log::error!("Request handle error: {}", e);
            return Ok(());
        }

        // 响应处理
        if let Err(e) = crate::handler::response_handle(session, resp, ctx).await {
            log::error!("Response handle error: {}", e);
            return Ok(());
        }

        ctx.insert_state("response:parts", SerdeParts::from(&*resp.deref_mut()));

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

        ctx.insert_state(
            HttpContext::RESPONSE_BODY_SIZE,
            body.as_ref().map(|b| b.len()),
        );

        Ok(None)
    }

    async fn logging(&self, session: &mut Session, err: Option<&Error>, ctx: &mut Self::CTX)
    where
        Self::CTX: Send + Sync,
    {
        // 日志记录
        crate::handler::log_handle(session, err, ctx, &self.args).await;
        // 清理
        crate::handler::cleanup_handle(session, ctx).await;
    }
}

pub fn from_status_code(stats_code: u16) -> SerdeParts {
    let resp = ServerSession::generate_error(stats_code);
    SerdeParts {
        method: None,
        status_code: Some(resp.status.clone()),
        uri: None,
        headers: Some(resp.headers.clone()),
        authority: None,
    }
}
