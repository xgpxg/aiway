use crate::components::ROUTER;
use crate::handler::{HandlerError, HandlerResult};
use aiway_protocol::context::{HttpContext, RequestExt};
use pingora::prelude::*;

pub async fn routing_handle(session: &mut Session, context: &mut HttpContext) -> HandlerResult<()> {
    // 匹配路由
    let router = ROUTER
        .get()
        .ok_or(HandlerError::new(500, "Router not initialized"))?;
    let head = session.req_header();
    let raw_host = head.get_host();
    let host = strip_port(&raw_host);
    let method = head.get_method().as_str();
    let path = head.get_path();
    let headers = head.all_request_headers();
    let query_str = head.query();
    let query = query_str.as_deref();
    let route = router
        .matches(host, method, &path, query, &headers)
        .ok_or(HandlerError::new(404, "Not Found"))?;
    context.set_route(route);

    Ok(())
}

/// 路由匹配时只匹配域名，忽略端口
fn strip_port(host: &str) -> &str {
    // IPv6 地址形如 [::1]:8080 或 [::1]
    if host.starts_with('[') {
        return match host.find("]:") {
            Some(idx) => &host[..=idx],
            None => host,
        };
    }
    match host.rfind(':') {
        Some(idx) => &host[..idx],
        None => host,
    }
}
