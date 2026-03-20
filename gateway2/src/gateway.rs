use crate::Args;
use aiway_plugin_v2::{ResponseHeader, async_trait};
use aiway_protocol::context::{HttpContext, RequestHeader, SessionExt};
use http::Uri;
use pingora::Error;
use pingora_core::prelude::HttpPeer;
use pingora_proxy::{ProxyHttp, Session};
use tokio_stream::StreamExt;

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
        // 提取内部请求上下文
        if let Err(e) = crate::handler::request_handle(session, ctx).await {
            session.respond_error_with_body(e.0, e.1.into()).await?;

            return Ok(true);
        }

        // 路由匹配
        if let Err(e) = crate::handler::routing_handle(session, ctx).await {
            session.respond_error_with_body(e.0, e.1.into()).await?;
            return Ok(true);
        }

        // 全局前置过滤器，可自由配置，串联执行，对整个网关生效，可做全局安全验证、监控、日志记录等。
        if let Err(e) = crate::handler::global_pre_filter(session, ctx).await {
            session.respond_error_with_body(e.0, e.1.into()).await?;

            return Ok(true);
        }

        // 鉴权，即验证API Key
        if let Err(e) = crate::handler::auth_handle(session, ctx).await {
            log::error!("Request handle error: {}", e);
            return Ok(true);
        }

        // 路由前置过滤器，可自由配置，串联执行，对单个路由生效，由于插件本身要求设计为无状态，所以，理论上各个路由的相同插件互不影响
        // 注意：是在路由匹配之后执行，因为要先匹配到路由，才能获取路由对应的插件，这点可能和命名有点歧义。
        if let Err(e) = crate::handler::pre_filter(session, ctx).await {
            log::error!("Request handle error: {}", e);
            return Ok(true);
        }

        // 负载均衡，通过路由配置对应的服务，进行负载，然后路由到具体的服务执行
        if let Err(e) = crate::handler::lb_handle(session, ctx).await {
            log::error!("Request handle error: {}", e);
            return Ok(true);
        }

        session.set_keepalive(Some(60));
        session.set_close_on_response_before_downstream_finish(false);

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
            session.respond_error_with_body(e.0, e.1.into()).await?;
            return Ok(());
        }

        // 前置安全校验
        if let Err(e) = crate::handler::security_check(session, ctx).await {
            session.respond_error_with_body(e.0, e.1.into()).await?;

            return Ok(());
        }
        Ok(())
    }

    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<(), Box<Error>> {
        let new_path = session.get_path();

        let query = upstream_request
            .uri
            .query()
            .map(|q| format!("?{}", q))
            .unwrap_or_default();
        let new_uri = format!("{}{}", new_path, query);

        upstream_request.set_uri(new_uri.parse().unwrap());

        Ok(())
    }

    async fn upstream_response_filter(
        &self,
        session: &mut Session,
        resp: &mut ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<(), Box<Error>> {
        log::info!("进入upstream_response_filter");
        // 路由后置过滤器，可自由配置，串联执行
        if let Err(e) = crate::handler::post_filter(session, resp, ctx).await {
            log::error!("Request handle error: {}", e);
            return Ok(());
        }
        // 全局后置过滤器，可自由配置，串联执行，对整个网关生效
        if let Err(e) = crate::handler::global_post_filter(session, resp, ctx).await {
            log::error!("Global post filter handle error: {}", e);
            return Ok(());
        }

        if let Err(e) = crate::handler::response_handle(session, resp, ctx).await {
            log::error!("Response handle error: {}", e);
            return Ok(());
        }

        // 日志记录
        crate::handler::log_handle(session, resp, ctx).await;

        // 清理
        crate::handler::cleanup_handle(session, ctx).await;
        Ok(())
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<()>
    where
        Self::CTX: Send + Sync,
    {
        println!("status: {}", upstream_response.status);
        Ok(())
    }
}
