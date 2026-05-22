use crate::init::alert_error;
use crate::{Args, handler};
use aiway_plugin::async_trait;
use aiway_protocol::common::header::Headers;
use aiway_protocol::context::parts::SerdeParts;
use aiway_protocol::context::{HttpContext, RequestExt};
use bytes::Bytes;
use handler::plugin::PluginType;
use handler::{HandlerError, plugin};
use http::Uri;
use pingora::http::{RequestHeader, ResponseHeader};
use pingora::prelude::{HttpPeer, ProxyHttp, Session};
use pingora::proxy::FailToProxy;
use pingora::{Error, ErrorType};
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

    /// 初始化网关上下文
    fn new_ctx(&self) -> Self::CTX {
        HttpContext::default()
    }

    /// 获取后端服务地址并连接
    async fn upstream_peer(
        &self,
        _: &mut Session,
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

        ctx.insert_any_state("host", host.to_string());

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
        plugin::run_on_request(PluginType::Global, session.req_header_mut(), ctx).await?;

        // 路由匹配
        handler::routing_handle(session, ctx).await?;

        // 鉴权
        handler::auth_handle(session, ctx).await?;

        // 执行路由请求阶段插件，可在此处修改http头部
        plugin::run_on_request(PluginType::Route, session.req_header_mut(), ctx).await?;

        // 负载均衡，查找服务实例
        handler::lb_handle(session, ctx).await?;

        //session.set_keepalive(Some(60));

        Ok(false)
    }

    /// 最早的filter，在此执行初始化、基础安全校验等
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
        handler::pre_handle(session, ctx);

        // 防火墙安全校验
        // 被防火墙拦截的请求不会记录日志
        handler::firewall_check(session, ctx).await?;

        Ok(())
    }

    /// 在向后端发送请求之前执行
    ///
    /// 这里修改Header将不生效
    async fn request_body_filter(
        &self,
        _: &mut Session,
        body: &mut Option<Bytes>,
        _end_of_stream: bool,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<()>
    where
        Self::CTX: Send + Sync,
    {
        // 执行插件
        plugin::run_on_request_body(PluginType::Route, body, ctx).await?;

        Ok(())
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        head: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<()>
    where
        Self::CTX: Send + Sync,
    {
        let host = ctx
            .get_any_state::<String>("host")
            .ok_or_else(|| Error::new_str("Host not found in context"))?;

        head.set_request_header(Headers::HOST, &host);

        head.remove_header(Headers::AUTHORIZATION);
        Ok(())
    }

    async fn upstream_response_filter(
        &self,
        session: &mut Session,
        resp: &mut ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<(), Box<Error>> {
        ctx.insert_state(
            HttpContext::RESPONSE_SERDE_PARTS,
            SerdeParts::from(resp.deref()),
        );

        // 响应处理
        handler::response_handle(session, resp, ctx).await;

        // 执行路由响应阶段插件
        plugin::run_on_response(PluginType::Route, resp, ctx).await?;

        // 执行全局响应阶段插件
        plugin::run_on_response(PluginType::Global, resp, ctx).await?;

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
        plugin::run_on_response_body(PluginType::Route, body, ctx)?;

        // 执行全局响应体阶段插件，可在此处修改响应body
        plugin::run_on_response_body(PluginType::Global, body, ctx)?;

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
        handler::log_handle(session, err, ctx, &self.args).await;
        // 清理
        handler::cleanup_handle(session, ctx).await;

        // 执行插件
        plugin::run_on_logging(ctx).await;
    }

    fn suppress_error_log(&self, _session: &Session, _ctx: &Self::CTX, error: &Error) -> bool {
        match error.etype {
            ErrorType::HTTPStatus(_) => true,
            _ => false,
        }
    }

    /// 统一响应错误信息
    async fn fail_to_proxy(
        &self,
        session: &mut Session,
        e: &Error,
        _ctx: &mut Self::CTX,
    ) -> FailToProxy
    where
        Self::CTX: Send + Sync,
    {
        match e.etype {
            ErrorType::HTTPStatus(status) => {
                if let Some(handler_err) = e
                    .cause
                    .as_ref()
                    .and_then(|cause| cause.downcast_ref::<HandlerError>())
                {
                    let _ = session
                        .respond_error_with_body(handler_err.0, handler_err.1.clone().into())
                        .await;

                    // 如果是502，则为网关内部错误，需要告警
                    if status == 502 {
                        log::error!("Gateway Proxy Error: {}", handler_err.1);
                        alert_error("Gateway Proxy Error", &handler_err.1);
                    }
                } else {
                    // downcast失败的，可能是框架原生错误，也需要告警
                    log::error!("Error: {:?}", e);
                    alert_error("Gateway Proxy Error", &e.to_string());
                }
            }
            _ => {
                // 非HTTPStatus的，会由Pingora自动打印日志，这里无需重复打印
                // 这里也需要告警
                alert_error("Gateway Proxy Error", &e.to_string());
            }
        };

        FailToProxy {
            error_code: 0,
            can_reuse_downstream: false,
        }
    }
}
