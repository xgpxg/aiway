use crate::handler::HttpResult;
use crate::report::STATE;
use aiway_protocol::context::{HttpContext, ResponseExt};
use pingora::prelude::{ResponseHeader, Session};

pub async fn response_handle(
    _session: &mut Session,
    resp: &mut ResponseHeader,
    ctx: &mut HttpContext,
) -> HttpResult<()> {
    if resp.is_sse() {
        STATE.inc_sse_connect_count(1);
        ctx.insert_state(HttpContext::IS_SSE, true);
    }
    if resp.is_ws() {
        STATE.inc_websocket_connect_count(1);
        ctx.insert_state(HttpContext::IS_WEBSOCKET, true);
    }

    Ok(())
}
