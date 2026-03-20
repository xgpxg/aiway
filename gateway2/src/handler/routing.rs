use http::HeaderMap;
use crate::components::ROUTER;
use crate::handler::{HttpError, HttpResult};
use aiway_protocol::context::{HttpContext, SessionExt};
use aiway_protocol::SV;
use pingora::prelude::*;

pub async fn routing_handle(
    session: &mut Session,
    context: &mut HttpContext,
) -> HttpResult<()> {
    // 匹配路由
    let router = ROUTER.get().ok_or(HttpError::new(500, "Router not initialized"))?;
    let host = session.get_host();
    let method = session.get_method().as_str();
    let path = session.get_path();
    let headers = session.all_request_headers();
    let query_str = session.query();
    let query = query_str.as_ref().map(|q| q.as_str());
    let route = router.matches(&host,method,&path,query,&headers).ok_or(HttpError::new(404, "Not Found"))?;
    context.request.route = SV::new(route.clone());

    Ok(())
}
