use crate::components::Servicer;
use crate::handler::lb::{LB_ATTEMPT, LB_CANDIDATES};
use crate::init::alert_error;
use crate::{Args, handler};
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
use pingora::upstreams::peer::ALPN;
use pingora::{Error, ErrorType};
use plugin_manager::Response as PluginResponse;
use plugin_manager::async_trait;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
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
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>, Box<Error>> {
        let backend_addr = ctx
            .get_routing_url()
            .ok_or_else(|| Error::new_str("Routing URL not found in context"))?
            .clone();

        // 转发到本地处理
        if backend_addr.ends_with(".sock") {
            let host = session.req_header().get_host();
            ctx.insert_any_state("host", host);

            return Ok(Box::new(HttpPeer::new_uds(
                &backend_addr,
                false,
                "".to_string(),
            )?));
        }

        // 候选的实例列表
        let candidates = ctx.get_any_state::<Vec<String>>(LB_CANDIDATES);

        // 无候选列表（未走 lb 的场景），直接用 routing_url
        let Some(candidates) = candidates else {
            return build_http_peer(ctx, &backend_addr);
        };

        // 获取当前尝试索引（fail_to_connect 中递增）
        let attempt = ctx
            .get_any_state::<AtomicUsize>(LB_ATTEMPT)
            .map_or(0, |a| a.load(Ordering::Relaxed));

        // 尝试次数小于候选列表长度时，继续尝试
        if attempt < candidates.len() {
            // 当前尝试的对应的索引处的实例
            let instance = &candidates[attempt];
            let peer = build_http_peer(ctx, instance)?;
            return Ok(peer);
        }

        // 所有候选均已尝试，还是没有可用的，返回异常
        Err(Error::new_str("All backend instances are unreachable"))
    }

    /// 在向后端发送请求之前
    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<bool, Box<Error>> {
        // 记录原始Parts
        ctx.insert_any_state(
            HttpContext::REQUEST_RAW_PARTS,
            SerdeParts::from(session.req_header().deref()),
        );

        // 执行全局请求阶段插件
        if let Some(resp) =
            plugin::run_on_request(PluginType::Global, session.req_header_mut(), ctx).await?
        {
            return respond_plugin(session, ctx, resp).await.map(|_| true);
        }

        // 路由匹配
        handler::routing_handle(session, ctx).await?;

        // 鉴权
        handler::auth_handle(session, ctx).await?;

        // 执行路由请求阶段插件，可在此处修改http头部
        if let Some(resp) =
            plugin::run_on_request(PluginType::Route, session.req_header_mut(), ctx).await?
        {
            return respond_plugin(session, ctx, resp).await.map(|_| true);
        }

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
        // 执行全局响应阶段插件
        plugin::run_on_request_body(PluginType::Global, body, ctx).await?;

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
        ctx.insert_any_state(
            HttpContext::RESPONSE_SERDE_PARTS,
            SerdeParts::from(&**resp.deref()),
        );

        // 响应处理
        handler::response_handle(session, resp, ctx).await;

        // 执行路由响应阶段插件
        if let Some(plugin_resp) = plugin::run_on_response(PluginType::Route, resp, ctx).await? {
            return respond_plugin(session, ctx, plugin_resp).await;
        }

        // 执行全局响应阶段插件
        if let Some(plugin_resp) = plugin::run_on_response(PluginType::Global, resp, ctx).await? {
            return respond_plugin(session, ctx, plugin_resp).await;
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
        // Pingora 的 response_body_filter 是同步接口，在此处桥接 async 插件
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // 执行路由响应体阶段插件，可在此处修改响应body
                plugin::run_on_response_body(PluginType::Route, body, ctx).await?;

                // 执行全局响应体阶段插件，可在此处修改响应body
                plugin::run_on_response_body(PluginType::Global, body, ctx).await?;

                Ok::<_, HandlerError>(())
            })
        })?;

        ctx.insert_any_state(
            HttpContext::RESPONSE_BODY_SIZE,
            body.as_ref().map(|b| b.len() as i64),
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
        matches!(error.etype, ErrorType::HTTPStatus(_))
    }

    /// 处理上游连接失败
    fn fail_to_connect(
        &self,
        _session: &mut Session,
        _peer: &HttpPeer,
        ctx: &mut Self::CTX,
        mut e: Box<Error>,
    ) -> Box<Error> {
        // 标记当前失败实例为不健康
        if let Some(addr) = ctx.get_routing_url() {
            Servicer::mark_unhealthy(addr);
        }

        // 获取候选实例
        let candidates: Option<Arc<Vec<String>>> = ctx.get_any_state(LB_CANDIDATES);

        // 获取或者初始化尝试次数
        let attempt = ctx.get_any_state::<AtomicUsize>(LB_ATTEMPT).or_else(|| {
            ctx.insert_any_state(LB_ATTEMPT, AtomicUsize::new(0));
            ctx.get_any_state::<AtomicUsize>(LB_ATTEMPT)
        });

        if let (Some(candidates), Some(attempt)) = (candidates, attempt) {
            let next = attempt.fetch_add(1, Ordering::Relaxed) + 1;
            if next < candidates.len() {
                log::warn!(
                    "Connection failed, will retry ({}/{})",
                    // +1 是排除首次调用
                    next + 1,
                    candidates.len()
                );
                e.set_retry(true);
            }
        }

        e
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

/// 从 URL 字符串构建 HttpPeer，同时设置上下文中的 host 头
fn build_http_peer(
    ctx: &mut HttpContext,
    addr: &str,
) -> pingora::Result<Box<HttpPeer>, Box<Error>> {
    let uri: Uri = addr
        .parse()
        .map_err(|_| Error::new_str("Failed to parse backend URI"))?;

    let host = uri
        .host()
        .ok_or_else(|| Error::new_str("Backend URI missing host"))?;

    let scheme = uri.scheme_str().unwrap_or("http");
    let is_tls = matches!(scheme, "https" | "wss");
    let is_grpc = matches!(scheme, "grpc");
    let port = uri.port_u16().unwrap_or(if is_tls { 443 } else { 80 });

    let host_string = host.to_string();
    ctx.insert_any_state("host", host_string.clone());

    let mut peer = Box::new(HttpPeer::new((host, port), is_tls, host_string));

    if is_grpc {
        peer.options.alpn = ALPN::H2;
    }

    Ok(peer)
}

/// 将插件主动响应发送给客户端
async fn respond_plugin(
    session: &mut Session,
    ctx: &mut HttpContext,
    resp: PluginResponse,
) -> pingora::Result<(), Box<Error>> {
    let mut response = ResponseHeader::build(resp.status, None)?;
    for (k, v) in &resp.headers {
        response.insert_header(k.clone(), v.as_bytes().to_vec())?;
    }

    // 记录插件主动响应到上下文，供日志阶段使用。
    // 插件主动响应不经过 upstream_response_filter，需要在此写入响应信息。
    ctx.insert_any_state(
        HttpContext::RESPONSE_SERDE_PARTS,
        SerdeParts::from(response.deref()),
    );
    ctx.insert_any_state(HttpContext::RESPONSE_BODY_SIZE, resp.body.len() as i64);

    session
        .write_response_header(Box::new(response), false)
        .await?;
    session
        .write_response_body(
            (!resp.body.is_empty()).then_some(Bytes::from(resp.body)),
            true,
        )
        .await?;
    Ok(())
}
