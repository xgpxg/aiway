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
    let host = head.get_host();
    let method = head.get_method().as_str();
    let path = head.get_path();
    let headers = head.all_request_headers();
    let query_str = head.query();
    let query = query_str.as_ref().map(|q| q.as_str());
    let route = router
        .matches(&host, method, &path, query, &headers)
        .ok_or(HandlerError::new(404, "Not Found"))?;
    context.set_route(route.clone().into());

    Ok(())
}
