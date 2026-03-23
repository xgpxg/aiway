use crate::handler::HttpResult;
use crate::report::STATE;
use aiway_protocol::context::{HttpContext, ResponseExt};
use pingora::prelude::{ResponseHeader, Session};

pub async fn response_handle(
    _session: &mut Session,
    resp: &mut ResponseHeader,
    _context: &mut HttpContext,
) -> HttpResult<()> {
    if resp.is_sse() {
        STATE.inc_sse_connect_count(1);
    }
    if resp.is_ws() {
        STATE.inc_websocket_connect_count(1);
    }

    Ok(())
}
